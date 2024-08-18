use std::vec::Drain;

use unscanny::Scanner;

use crate::error::SyntaxError;
use crate::go_syntax::EscapeContext;
use crate::{go_syntax, SyntaxKind, TextRange, TextSize};

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

impl<'s> Lexer<'s> {
    pub fn new(input: &'s str) -> Lexer<'s> {
        Lexer {
            s: Scanner::new(input),
            mode: LexMode::Text,
            errors: Vec::with_capacity(1),
        }
    }

    pub fn input(&self) -> &'s str {
        self.s.string()
    }

    pub fn done(&self) -> bool {
        self.s.done()
    }

    pub fn peek_next_satisfying<P>(&mut self, pred: P) -> SyntaxKind
    where
        P: Fn(SyntaxKind) -> bool,
    {
        let orig_pos = self.s.cursor();
        let orig_mode = self.mode;
        let orig_error_count = self.errors.len();
        loop {
            let token = self.next();
            if pred(token) {
                self.s.jump(orig_pos);
                self.mode = orig_mode;
                self.errors.truncate(orig_error_count);
                break token;
            }
        }
    }

    pub fn cursor(&self) -> TextSize {
        TextSize::new(self.s.cursor() as u32)
    }

    /// Extract the syntax errors accumulated so far.
    pub fn drain_errors(&mut self) -> Drain<'_, SyntaxError> {
        self.errors.drain(..)
    }
}

impl Lexer<'_> {
    fn error(&mut self, message: impl Into<String>, range: TextRange) {
        self.errors.push(SyntaxError::new(message.into(), range));
    }

    fn error_at(&mut self, pos: TextSize, message: impl Into<String>) {
        self.error(message, TextRange::empty(pos));
    }

    fn error_from(&mut self, pos: TextSize, message: impl Into<String>) {
        self.error(message, TextRange::new(pos, self.cursor()));
    }
}

impl Lexer<'_> {
    #[allow(clippy::should_implement_trait)]
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
        let start = self.cursor();
        let Some(c) = self.s.eat() else {
            return SyntaxKind::Eof;
        };
        match c {
            '/' if self.s.eat_if('*') => self.comment(start),
            ' ' if self.s.eat_if("-}}") => {
                self.mode = LexMode::Text;
                SyntaxKind::TrimmedRightDelim
            }
            c if upstream_compat::is_space(c) => self.whitespace(),
            ',' => SyntaxKind::Comma,
            '=' => SyntaxKind::Eq,
            ':' if self.s.eat_if('=') => SyntaxKind::ColonEq,
            '|' => SyntaxKind::Pipe,
            '.' if self.s.at(char::is_ascii_digit) => {
                self.s.uneat();
                self.number()
            }
            '.' if self.s.eat_if(char::is_alphanumeric) => {
                self.s.eat_while(char::is_alphanumeric);
                SyntaxKind::Field
            }
            '.' => SyntaxKind::Dot,
            '(' => SyntaxKind::LeftParen,
            ')' => SyntaxKind::RightParen,
            '$' => self.var(),
            '"' => self.interpreted_string(start),
            '`' => self.raw_string(start),
            '\'' => self.char_literal(start),
            '+' | '-' | '0'..='9' => {
                self.s.uneat();
                self.number()
            }
            '}' if self.s.eat_if('}') => {
                self.mode = LexMode::Text;
                SyntaxKind::RightDelim
            }
            c if c.is_alphanumeric() => self.ident(start),
            _ => {
                self.error_from(start, format!("invalid character {c:?} in action"));
                SyntaxKind::InvalidCharInAction
            }
        }
    }

    fn comment(&mut self, start: TextSize) -> SyntaxKind {
        self.s.eat_until("*/");
        if !self.s.eat_if("*/") {
            self.error_from(start, "unclosed comment");
        }
        SyntaxKind::Comment
    }

    fn whitespace(&mut self) -> SyntaxKind {
        self.s.eat_whitespace();
        SyntaxKind::Whitespace
    }

    fn ident(&mut self, start: TextSize) -> SyntaxKind {
        self.s.eat_while(upstream_compat::is_alphanumeric);
        let ident = self.s.from(start.into());
        SyntaxKind::from_ident(ident).unwrap_or(SyntaxKind::Ident)
    }

    fn var(&mut self) -> SyntaxKind {
        self.s.eat_while(upstream_compat::is_alphanumeric);
        SyntaxKind::Var
    }

    fn interpreted_string(&mut self, start: TextSize) -> SyntaxKind {
        while !self.done() && !self.s.at(['"', '\n']) {
            let in_escape = self.s.eat() == Some('\\');
            if in_escape {
                self.s.eat();
            }
        }

        if self.done() {
            self.error_from(start, "unclosed string");
        } else if self.s.at('\n') {
            self.error_at(self.cursor(), "unexpected newline in string");
        } else if self.s.eat_if('"') {
            // validate escape sequences
            self.errors.extend(
                go_syntax::iter_escape_sequences(self.s.from(start.into()), EscapeContext::StringLiteral).filter_map(
                    |(range, result)| match result {
                        Ok(_) => None,
                        Err(err) => Some(SyntaxError::new(err.to_string(), range)),
                    },
                ),
            );
        }

        SyntaxKind::InterpretedString
    }

    fn raw_string(&mut self, start: TextSize) -> SyntaxKind {
        self.s.eat_until('`');
        if !self.s.eat_if('`') {
            self.error_from(start, "unclosed raw string");
        }
        SyntaxKind::RawString
    }

    fn char_literal(&mut self, start: TextSize) -> SyntaxKind {
        let Some(c) = self.s.eat() else {
            self.error_at(self.cursor(), "expected character after `'`");
            return SyntaxKind::Char;
        };

        if c == '\\' {
            match self.s.eat() {
                Some(c) => {
                    let result = go_syntax::scan_escape_sequence(&mut self.s, c, EscapeContext::CharacterLiteral);
                    if let Err(err) = result {
                        self.error_from(start, err.to_string());
                    }
                }
                None => self.error_at(self.cursor(), "expected character after `\\`"),
            }
        }

        if !self.s.eat_if('\'') {
            self.error_at(self.cursor(), "expected `'` closing character literal");
        }
        SyntaxKind::Char
    }

    fn number(&mut self) -> SyntaxKind {
        let start = self.cursor();
        self.s.eat_if(['+', '-']);

        // scan prefix
        let base = go_syntax::scan_numeric_base_prefix(&mut self.s).unwrap_or(10);

        // scan integer part
        self.scan_digits(if base == 16 {
            char::is_ascii_alphanumeric
        } else {
            char::is_ascii_digit
        });

        let interpret_as_float = if base == 10 {
            // scan decimal part
            let has_decimal = self.s.eat_if('.');
            if has_decimal {
                self.scan_digits(char::is_ascii_digit);
            }

            // scan exponent
            let has_exp = self.s.eat_if(['e', 'E']);
            if has_exp {
                self.s.eat_if(['+', '-']);
                self.scan_digits(char::is_ascii_digit);
            }

            has_decimal || has_exp
        } else {
            false
        };

        if interpret_as_float {
            if go_syntax::parse_float(self.s.from(start.into())).is_err() {
                self.error_from(start, "invalid number syntax");
            }
            SyntaxKind::Float
        } else {
            if go_syntax::parse_int(self.s.from(start.into())).is_err() {
                self.error_from(start, "invalid number syntax");
            }
            SyntaxKind::Int
        }
    }

    fn scan_digits<F>(&mut self, allow: F)
    where
        F: Fn(&char) -> bool,
    {
        self.s.eat_while(|c| c == '_' || allow(&c));
    }
}

mod upstream_compat {
    use unic_ucd_category::GeneralCategory;

    /// Whether c is a space character, according to the original text/template
    /// parser in Go.
    ///
    /// Refer to the `isSpace` function below:
    /// https://github.com/golang/go/blob/master/src/text/template/parse/lex.go#L671
    pub(super) fn is_space(c: char) -> bool {
        matches!(c, ' ' | '\t' | '\r' | '\n')
    }

    /// Whether c is alphanumeric according to the original text/template parser
    /// in Go.
    ///
    /// Refer to the `isAlphaNumeric` function below:
    /// https://github.com/golang/go/blob/master/src/text/template/parse/lex.go#L676
    pub(super) fn is_alphanumeric(c: char) -> bool {
        match c {
            '_' | '0'..='9' | 'a'..='z' | 'A'..='Z' => true,
            c if c > '\x7f' => {
                let category = GeneralCategory::of(c);
                category.is_letter() || category == GeneralCategory::DecimalNumber
            }
            _ => false,
        }
    }
}
