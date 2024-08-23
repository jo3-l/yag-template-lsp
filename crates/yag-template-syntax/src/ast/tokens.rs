use std::borrow::Cow;

use unscanny::Scanner;

use super::macros::define_ast_token;
use crate::ast::AstToken;
use crate::go_syntax::EscapeContext;
use crate::{go_syntax, SyntaxKind, SyntaxToken};

define_ast_token! {
    pub struct Text;
}

impl Text {
    pub fn get(&self) -> &str {
        self.syntax.text()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct LeftDelim {
    syntax: SyntaxToken,
}

impl AstToken for LeftDelim {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
    }

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized,
    {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

impl LeftDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TrimmedLeftDelim
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct RightDelim {
    syntax: SyntaxToken,
}

impl AstToken for RightDelim {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::RightDelim | SyntaxKind::TrimmedLeftDelim)
    }

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized,
    {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

impl RightDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TrimmedRightDelim
    }
}

define_ast_token! {
    pub struct Ident;
}

impl Ident {
    pub fn get(&self) -> &str {
        self.syntax.text()
    }
}

define_ast_token! {
    pub struct Field;
}

impl Field {
    pub fn name(&self) -> Option<&str> {
        self.syntax.text().strip_prefix('.').filter(|name| !name.is_empty())
    }
}

define_ast_token! {
    pub struct Var;
}

impl Var {
    pub fn name(&self) -> &str {
        self.syntax.text()
    }
}

define_ast_token! {
    pub struct Bool;
}

impl Bool {
    pub fn get(&self) -> bool {
        self.syntax.text() == "true"
    }
}

define_ast_token! {
    pub struct Int;
}

impl Int {
    pub fn get(&self) -> Option<i64> {
        go_syntax::parse_int(self.syntax.text()).ok()
    }
}

define_ast_token! {
    pub struct Float;
}

impl Float {
    pub fn get(&self) -> Option<f64> {
        go_syntax::parse_float(self.syntax.text()).ok()
    }
}

define_ast_token! {
    pub struct Char;
}

impl Char {
    pub fn get(&self) -> Option<char> {
        let mut s = Scanner::new(self.syntax.text());
        s.eat_if('\'');
        match s.eat() {
            Some('\\') => {
                let after_slash = s.eat()?;
                go_syntax::scan_escape_sequence(&mut s, after_slash, EscapeContext::CharacterLiteral).ok()
            }
            Some(c) => Some(c),
            None => None,
        }
    }
}

define_ast_token! {
    pub struct Nil;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StringLiteral {
    Interpreted(InterpretedString),
    Raw(RawString),
}

impl AstToken for StringLiteral {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::InterpretedString | SyntaxKind::RawString)
    }

    fn cast(syntax: SyntaxToken) -> Option<Self> {
        match syntax.kind() {
            SyntaxKind::InterpretedString => InterpretedString::cast(syntax).map(Self::Interpreted),
            SyntaxKind::RawString => RawString::cast(syntax).map(Self::Raw),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        match self {
            Self::Interpreted(v) => &v.syntax,
            Self::Raw(v) => &v.syntax,
        }
    }
}

define_ast_token! {
    pub struct InterpretedString;
}

impl InterpretedString {
    pub fn get(&self) -> Cow<str> {
        let content = strip_quotes(self.syntax.text(), '"');
        go_syntax::interpret_string_content(content)
    }
}

define_ast_token! {
    pub struct RawString;
}

impl RawString {
    pub fn get(&self) -> &str {
        strip_quotes(self.syntax.text(), '`')
    }
}

fn strip_quotes(s: &str, quote_char: char) -> &str {
    let without_leading = s.strip_prefix(quote_char).unwrap_or(s);
    without_leading.strip_suffix(quote_char).unwrap_or(without_leading)
}
