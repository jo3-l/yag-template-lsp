mod actions;
mod demo;
mod expr;
mod token_set;

use actions::text_or_action;
use drop_bomb::DropBomb;
pub(crate) use rowan::Checkpoint;
use rowan::{GreenNode, GreenNodeBuilder};

use crate::error::SyntaxError;
use crate::lexer::Lexer;
use crate::parser::token_set::TokenSet;
use crate::{SyntaxKind, TextRange, TextSize};

pub fn parse(input: &str) -> Parse {
    let mut p = Parser::new(input);
    let root = p.start(SyntaxKind::Root);
    while !p.at_eof() {
        text_or_action(&mut p);
    }
    root.complete(&mut p);
    p.finish()
}

#[derive(Debug, Clone)]
pub struct Parse {
    pub root: GreenNode,
    pub errors: Vec<SyntaxError>,
}

#[derive(Debug)]
pub(crate) struct Parser<'s> {
    green: GreenNodeBuilder<'static>,
    lexer: Lexer<'s>,
    errors: Vec<SyntaxError>,
    cur_start: TextSize,
    cur: SyntaxKind,
}

impl<'s> Parser<'s> {
    pub(crate) fn new(input: &'s str) -> Parser {
        let mut lexer = Lexer::new(input);
        let current = lexer.next();
        Parser {
            green: GreenNodeBuilder::new(),
            lexer,
            errors: Vec::new(),
            cur_start: TextSize::new(0),
            cur: current,
        }
    }

    pub(crate) fn finish(self) -> Parse {
        Parse {
            root: self.green.finish(),
            errors: self.errors,
        }
    }
}

pub(crate) struct Marker(DropBomb);

impl Marker {
    /// Mark the subtree starting at the beginning of the marker as complete.
    pub(crate) fn complete(mut self, p: &mut Parser) {
        self.0.defuse();
        p.green.finish_node();
    }
}

// Methods for manipulating the parse tree.
impl Parser<'_> {
    /// Begin a new subtree of the given kind, containing all subsequent nodes
    /// until `Marker::complete` is called.
    #[must_use]
    pub(crate) fn start(&mut self, kind: SyntaxKind) -> Marker {
        self.green.start_node(kind.into());
        Marker(DropBomb::new(
            "all calls to Parser::start() must have corresponding Parser::complete() call",
        ))
    }

    /// Create a checkpoint which can be used (with the aid of [Parser::wrap])
    /// to retroactively wrap nodes in a subtree. It is useful when the precise
    /// type of the parent node is not known until further parsing occurs.
    pub(crate) fn checkpoint(&mut self) -> Checkpoint {
        self.green.checkpoint()
    }

    /// Wrap all nodes between the checkpoint and the current position within a
    /// new parent node of the given kind.
    pub(crate) fn wrap(&mut self, c: Checkpoint, kind: SyntaxKind) {
        self.green.start_node_at(c, kind.into());
        self.green.finish_node();
    }
}

// Methods for accessing and consuming tokens. In general, all methods ignore
// trivia but not whitespace (which can be significant) unless stated otherwise.
impl<'s> Parser<'s> {
    pub(crate) fn at_eof(&self) -> bool {
        self.cur == SyntaxKind::Eof
    }

    pub(crate) fn cur(&self) -> SyntaxKind {
        self.cur
    }

    pub(crate) fn peek_ignore_space(&mut self) -> SyntaxKind {
        let checkpoint = self.lexer.checkpoint();
        loop {
            let token = self.lexer.next();
            if !matches!(token, SyntaxKind::Whitespace | SyntaxKind::Comment) {
                self.lexer.restore(checkpoint);
                break token;
            }
        }
    }

    pub(crate) fn cur_start(&self) -> TextSize {
        self.cur_start
    }

    pub(crate) fn cur_end(&self) -> TextSize {
        self.lexer.cursor()
    }

    pub(crate) fn cur_range(&self) -> TextRange {
        TextRange::new(self.cur_start(), self.cur_end())
    }

    pub(crate) fn cur_text(&self) -> &'s str {
        &self.lexer.input()[self.cur_range()]
    }

    pub(crate) fn at(&self, pat: impl TokenPattern) -> bool {
        pat.matches(self.cur)
    }

    pub(crate) fn at_ignore_space(&mut self, pat: impl TokenPattern) -> bool {
        if self.at(SyntaxKind::Whitespace) {
            pat.matches(self.peek_ignore_space())
        } else {
            self.at(pat)
        }
    }

    pub(crate) fn eat_if(&mut self, pat: impl TokenPattern) -> bool {
        let at = self.at(pat);
        if at {
            self.eat();
        }
        at
    }

    /// Add the current token to the parse tree if not at EOF, then advance to
    /// the next non-trivia token.
    pub(crate) fn eat(&mut self) {
        if !self.at_eof() {
            self.only_eat_cur_token();
        }
        self.eat_trivia();
    }

    /// Eat all leading whitespace and trivia and report whether any whitespace
    /// was consumed.
    pub(crate) fn eat_whitespace(&mut self) -> bool {
        let at = self.at(SyntaxKind::Whitespace);
        while self.at(SyntaxKind::Whitespace) {
            self.eat();
        }
        at
    }

    fn eat_trivia(&mut self) {
        while self.at(SyntaxKind::Comment) {
            self.only_eat_cur_token();
        }
    }

    /// Eat only the current token; do not skip trailing trivia.
    pub(crate) fn only_eat_cur_token(&mut self) {
        self.green.token(self.cur.into(), self.cur_text());
        self.cur_start = self.lexer.cursor();
        self.cur = self.lexer.next();
        if let Some(err) = self.lexer.take_error() {
            self.errors.push(err);
        }
    }
}

// Methods for error reporting and recovery.
impl Parser<'_> {
    pub(crate) fn wrap_err<F, R>(&mut self, parser: F, err_msg: impl Into<String>)
    where
        F: FnOnce(&mut Parser) -> R,
    {
        let error = self.start(SyntaxKind::Error);
        let start = self.cur_start;
        parser(self);
        error.complete(self);
        self.error(err_msg, TextRange::new(start, self.cur_start));
    }

    /// Eat leading whitespace and produce an error if none was found.
    pub(crate) fn expect_whitespace(&mut self, context: &str) {
        if !self.eat_whitespace() {
            self.error_here(format!("expected space {context}; found {}", self.cur))
        }
    }

    /// Eat the current token if it matches `kind`, otherwise produce an error
    /// and continue via [Parser::err_recover]. The boolean result indicates
    /// whether the token matched.
    pub(crate) fn expect_recover(&mut self, kind: SyntaxKind, recover: TokenSet) -> bool {
        let at = self.at(kind);
        if at {
            self.eat();
        } else {
            self.err_recover(format!("expected {}", kind), recover);
        }
        at
    }

    /// Unconditionally eat the current token, producing an error if it does not
    /// match `kind`. For improved error recovery, prefer
    /// [Parser::expect_recover] when possible; see the documentation for
    /// [Parser::err_recover] for further explanation.
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> bool {
        let at = self.at(kind);
        if at {
            self.eat();
        } else {
            self.err_and_eat(format!("expected {}", kind));
        }
        at
    }

    pub(crate) fn assert(&mut self, kind: SyntaxKind) {
        assert_eq!(self.cur, kind);
        self.eat();
    }

    /// Produce an error and eat the current token if it is not in the
    /// `recovery` set.
    ///
    /// A careful choice of `recovery` can minimize the impact of syntax errors
    /// on the parse tree produced for subsequent, otherwise correct, input. For
    /// instance, generally one will want to mark both left action delimiters
    /// (`{{`, `{{- `) as recoverable so they are not consumed upon error. This
    /// way, given an input such as
    /// ```text
    /// {{$x := {{add 1 2}}
    /// ```
    /// when the parser encounters the `{{` (where an expression is expected),
    /// an error is produced but the `{{` is not consumed, allowing `{{add 1
    /// 2}}` to be parsed completely.
    pub(crate) fn err_recover(&mut self, message: impl Into<String>, recovery: TokenSet) {
        if self.at(recovery) {
            self.error(message, TextRange::empty(self.cur_start));
        } else {
            self.err_and_eat(message);
        }
    }

    /// Create an error node and consume the current token if not at EOF. When
    /// possible, prefer to call [Parser::err_recover] so that the impact of the
    /// error on parsing of subsequent correct input can be minimized.
    pub(crate) fn err_and_eat(&mut self, message: impl Into<String>) {
        self.error(message, self.cur_range());
        if !self.at(SyntaxKind::Eof) {
            let error = self.start(SyntaxKind::Error);
            self.eat();
            error.complete(self);
        }
    }

    /// Emit a syntax error at the given span without touching the current token
    /// or the parse tree.
    ///
    /// Callers should take special care to ensure that the parser does not get
    /// stuck if this method is called directly.
    pub(crate) fn error(&mut self, message: impl Into<String>, range: TextRange) {
        self.errors.push(SyntaxError::new(message, range));
    }

    /// Call [Parser::error] at the range of the current token.
    pub(crate) fn error_here(&mut self, message: impl Into<String>) {
        self.error(message, self.cur_range())
    }
}

pub(crate) trait TokenPattern {
    fn matches(self, kind: SyntaxKind) -> bool;
}

impl TokenPattern for SyntaxKind {
    fn matches(self, kind: SyntaxKind) -> bool {
        self == kind
    }
}

impl TokenPattern for TokenSet {
    fn matches(self, kind: SyntaxKind) -> bool {
        self.contains(kind)
    }
}
