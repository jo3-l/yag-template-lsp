use rowan::{GreenNode, GreenNodeBuilder};

use crate::error::SyntaxError;
use crate::kind::SyntaxKind;
use crate::lexer::Lexer;
use crate::token_set::TokenSet;
use crate::{TextRange, TextSize};

#[derive(Debug, Clone)]
pub struct Parse {
    pub root: GreenNode,
    pub errors: Vec<SyntaxError>,
}

#[derive(Debug)]
pub(crate) struct Parser<'s> {
    events: Vec<Event<'s>>,
    lexer: Lexer<'s>,
    errors: Vec<SyntaxError>,
    cur_start: TextSize,
    cur: SyntaxKind,
    had_leading_whitespace: bool,
}

impl<'s> Parser<'s> {
    pub(crate) fn new(input: &'s str) -> Parser {
        let mut lexer = Lexer::new(input);
        let current = lexer.next();
        Parser {
            events: Vec::new(),
            lexer,
            errors: Vec::new(),
            cur_start: TextSize::new(0),
            cur: current,
            had_leading_whitespace: false,
        }
    }

    pub(crate) fn finish(self) -> Parse {
        Parse {
            root: process(self.events),
            errors: self.errors,
        }
    }
}

#[derive(Debug)]
enum Event<'s> {
    Abandoned,
    Start(Option<SyntaxKind>),
    Finish,
    Leaf { kind: SyntaxKind, text: &'s str },
}

fn process(events: Vec<Event>) -> GreenNode {
    let mut builder = GreenNodeBuilder::new();
    for event in events {
        match event {
            Event::Abandoned => (),
            Event::Start(None) => {
                panic!("unexpected pending start event (must be completed or abandoned)")
            }
            Event::Start(Some(kind)) => builder.start_node(kind.into()),
            Event::Finish => builder.finish_node(),
            Event::Leaf { kind, text } => builder.token(kind.into(), text),
        }
    }
    builder.finish()
}

#[derive(Debug)]
pub(crate) struct Marker(usize);

// Manipulating the parse tree.
impl Parser<'_> {
    #[must_use]
    pub(crate) fn marker(&mut self) -> Marker {
        let idx = self.events.len();
        self.events.push(Event::Start(None));
        Marker(idx)
    }

    pub(crate) fn abandon(&mut self, marker: Marker) {
        self.events[marker.0] = Event::Abandoned;
    }

    pub(crate) fn wrap(&mut self, from: Marker, kind: SyntaxKind) {
        self.events[from.0] = Event::Start(Some(kind));
        self.events.push(Event::Finish);
    }

    pub(crate) fn wrap_within(&mut self, from: Marker, to: Marker, kind: SyntaxKind) {
        self.events[from.0] = Event::Start(Some(kind));
        self.events[to.0] = Event::Finish;
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

    /// Is the current token preceded by whitespace?
    pub(crate) fn had_leading_whitespace(&self) -> bool {
        self.had_leading_whitespace
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
        self.events.push(Event::Leaf {
            kind: self.cur,
            text: self.cur_text(),
        });
        self.had_leading_whitespace = self.cur == SyntaxKind::Whitespace;
        self.cur_start = self.lexer.cursor();
        self.cur = self.lexer.next();
    }
}

// Error reporting.
impl Parser<'_> {
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
