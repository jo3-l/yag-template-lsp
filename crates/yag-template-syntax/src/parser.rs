use drop_bomb::DropBomb;
use rowan::{Checkpoint, GreenNode, GreenNodeBuilder};

use crate::error::SyntaxError;
use crate::lexer::Lexer;
use crate::token_set::TokenSet;
use crate::{SyntaxKind, TextRange, TextSize};

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
    preceded_by_whitespace: bool,
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
            preceded_by_whitespace: false,
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
    pub(crate) fn complete(mut self, p: &mut Parser) {
        self.0.defuse();
        p.green.finish_node();
    }
}

// Manipulating the parse tree.
impl Parser<'_> {
    #[must_use]
    pub(crate) fn start(&mut self, kind: SyntaxKind) -> Marker {
        self.green.start_node(kind.into());
        Marker(DropBomb::new(
            "all calls to Parser::start() must have corresponding Parser::complete()",
        ))
    }

    pub(crate) fn checkpoint(&mut self) -> Checkpoint {
        self.green.checkpoint()
    }

    pub(crate) fn wrap(&mut self, c: Checkpoint, kind: SyntaxKind) {
        self.green.start_node_at(c, kind.into());
        self.green.finish_node();
    }
}

// Accessing and consuming tokens.
impl<'s> Parser<'s> {
    pub(crate) fn done(&self) -> bool {
        self.cur == SyntaxKind::Eof
    }

    pub(crate) fn cur(&self) -> SyntaxKind {
        self.cur
    }

    pub(crate) fn peek(&mut self) -> SyntaxKind {
        let checkpoint = self.lexer.checkpoint();
        loop {
            let token = self.lexer.next();
            if !token.is_trivia() {
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

    pub(crate) fn preceded_by_whitespace(&self) -> bool {
        self.preceded_by_whitespace
    }

    pub(crate) fn at(&self, pat: impl TokenPattern) -> bool {
        pat.matches(self.cur)
    }

    pub(crate) fn at2(&mut self, pat0: impl TokenPattern, pat1: impl TokenPattern) -> bool {
        pat0.matches(self.cur) && pat1.matches(self.peek())
    }

    pub(crate) fn eat_if(&mut self, pat: impl TokenPattern) -> bool {
        let at = self.at(pat);
        if at {
            self.eat();
        }
        at
    }

    pub(crate) fn assert(&mut self, pat: impl TokenPattern) {
        assert!(pat.matches(self.cur));
        self.eat();
    }

    /// Add the current token to the parse tree, then call [Parser::skip] to
    /// move to the next non-trivia token.
    pub(crate) fn eat(&mut self) {
        self.eat_one();
        self.skip_trivia();
    }

    pub(crate) fn skip_trivia(&mut self) {
        while self.at(SyntaxKind::Comment) || self.at(SyntaxKind::Whitespace) {
            self.eat_one();
        }
    }

    fn eat_one(&mut self) {
        self.green.token(self.cur.into(), self.cur_text());
        self.preceded_by_whitespace = self.cur == SyntaxKind::Whitespace;
        self.cur_start = self.lexer.cursor();
        self.cur = self.lexer.next();
    }
}

// Error reporting.
impl Parser<'_> {
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> bool {
        let at = self.at(kind);
        if at {
            self.eat();
        } else {
            self.error_and_eat(format!("expected {}", kind.name()));
        }
        at
    }

    pub(crate) fn expect_with_recover(&mut self, kind: SyntaxKind, recoverable: TokenSet) -> bool {
        let at = self.at(kind);
        if at {
            self.eat();
        } else {
            self.error_with_recover(format!("expected {}", kind.name()), recoverable);
        }
        at
    }

    /// Create an error node and consume the next token.
    pub(crate) fn error_and_eat(&mut self, message: impl Into<String>) {
        self.error(message, self.cur_range());
        self.eat();
    }

    pub(crate) fn error_with_recover(&mut self, message: impl Into<String>, recoverable: TokenSet) {
        if self.at(recoverable) {
            self.error(message, TextRange::empty(self.cur_start));
        } else {
            self.error_and_eat(message);
        }
    }

    /// Emit a syntax error at the given span without touching the current token
    /// or the parse tree.
    pub(crate) fn error(&mut self, message: impl Into<String>, range: TextRange) {
        self.errors.push(SyntaxError::new(message, range));
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
