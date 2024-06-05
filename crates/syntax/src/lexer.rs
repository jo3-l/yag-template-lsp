use unscanny::Scanner;

use crate::error::SyntaxError;
use crate::kind::SyntaxKind;
use crate::span::Span;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum LexMode {
    Action,
    Text,
}

#[derive(Debug)]
pub struct Lexer<'s> {
    s: Scanner<'s>,
    mode: LexMode,
    errors: Vec<SyntaxError>,
}

#[derive(Debug)]
pub struct Checkpoint {
    cursor: usize,
    mode: LexMode,
    error_count: usize,
}

impl<'s> Lexer<'s> {
    pub fn new(input: &'s str) -> Lexer<'s> {
        Lexer {
            s: Scanner::new(input),
            mode: LexMode::Text,
            errors: Vec::new(),
        }
    }

    pub fn input(&self) -> &'s str {
        self.s.string()
    }

    pub fn done(&self) -> bool {
        self.s.done()
    }

    pub fn cursor(&self) -> usize {
        self.s.cursor()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            cursor: self.s.cursor(),
            mode: self.mode,
            error_count: self.errors.len(),
        }
    }

    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.s.jump(checkpoint.cursor);
        self.mode = checkpoint.mode;
        self.errors.truncate(checkpoint.error_count);
    }

    pub fn finish(mut self) -> Vec<SyntaxError> {
        while !self.done() {
            self.next();
        }
        self.errors
    }
}

impl Lexer<'_> {
    fn error(&mut self, message: impl Into<String>, span: Span) {
        self.errors.push(SyntaxError::new(message, span));
    }

    fn error_at_offset(&mut self, message: impl Into<String>, offset: usize) {
        self.errors
            .push(SyntaxError::new(message, Span::empty(offset)));
    }
}

impl Lexer<'_> {
    pub fn next(&mut self) -> SyntaxKind {
        if self.done() {
            SyntaxKind::Eof
        } else if self.s.eat_if("{{") {
            self.mode = LexMode::Action;
            if self.s.eat_if("- ") {
                // space is required: just '{{-' might be a negative number
                SyntaxKind::TrimmedLeftDelim
            } else {
                SyntaxKind::LeftDelim
            }
        } else {
            match self.mode {
                LexMode::Action => self.action(),
                LexMode::Text => self.text(),
            }
        }
    }

    fn text(&mut self) -> SyntaxKind {
        self.s.eat_until("{{");
        SyntaxKind::Text
    }

    fn action(&mut self) -> SyntaxKind {
        let start = self.s.cursor();
        let Some(c) = self.s.eat() else {
            return SyntaxKind::Eof;
        };
        match c {
            '/' if self.s.eat_if('*') => self.comment(start),
            ' ' if self.s.eat_if("-}}") => {
                self.mode = LexMode::Text;
                SyntaxKind::TrimmedRightDelim
            }
            _ if c.is_whitespace() => self.whitespace(),
            '=' => SyntaxKind::Eq,
            ':' if self.s.eat_if('=') => SyntaxKind::ColonEq,
            '$' => self.var(),
            '+' | '-' | '0'..='9' => {
                self.s.uneat();
                self.number()
            }
            _ if c.is_alphanumeric() => self.ident(start),
            '}' if self.s.eat_if('}') => {
                self.mode = LexMode::Text;
                SyntaxKind::RightDelim
            }
            _ => {
                self.error(
                    format!("unexpected character {c:?} in action"),
                    Span::new(start, self.s.cursor()),
                );
                SyntaxKind::Error
            }
        }
    }

    fn comment(&mut self, start: usize) -> SyntaxKind {
        self.s.eat_until("*/");
        if !self.s.eat_if("*/") {
            self.error("unclosed comment", Span::new(start, self.s.cursor()));
        }
        SyntaxKind::Comment
    }

    fn whitespace(&mut self) -> SyntaxKind {
        self.s.eat_whitespace();
        SyntaxKind::Whitespace
    }

    fn number(&mut self) -> SyntaxKind {
        self.s.eat_if(['+', '-']);
        let radix = if self.s.eat_if("0x") || self.s.eat_if("0X") {
            16
        } else if self.s.eat_if("0o") || self.s.eat_if("0O") {
            8
        } else if self.s.eat_if("0b") || self.s.eat_if("0B") {
            2
        } else {
            10
        };
        self.s.eat_while(|c: &char| c.to_digit(radix).is_some());
        SyntaxKind::Int
    }

    fn ident(&mut self, start: usize) -> SyntaxKind {
        self.s.eat_while(char::is_alphanumeric);
        let ident = self.s.from(start);
        SyntaxKind::try_from_keyword(ident).unwrap_or(SyntaxKind::Ident)
    }

    fn var(&mut self) -> SyntaxKind {
        self.s.eat_while(char::is_alphanumeric);
        SyntaxKind::Var
    }
}
