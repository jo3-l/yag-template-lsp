use std::borrow::Cow;

use unscanny::Scanner;

use crate::ast::AstElement;
use crate::go_lit_syntax::EscapeContext;
use crate::{go_lit_syntax, SyntaxElement, SyntaxKind, SyntaxToken};

macro_rules! define_token {
    ($(#[$attr:meta])* $name:ident($pat:pat)) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name {
            syntax: SyntaxToken,
        }

        impl AstElement for $name {
            fn cast(element: SyntaxElement) -> Option<Self> {
                element.into_token().and_then(|token| {
                    if matches!(token.kind(), $pat) {
                        Some(Self { syntax: token })
                    } else {
                        None
                    }
                })
            }
        }

        impl $name {
            pub fn syntax(&self) -> &SyntaxToken {
                &self.syntax
            }
        }
    };
}

define_token! {
    Text(SyntaxKind::Text)
}

impl Text {
    pub fn get(&self) -> &str {
        self.syntax.text()
    }
}

define_token! {
    LeftDelim(SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
}

impl LeftDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TrimmedLeftDelim
    }
}

define_token! {
    RightDelim(SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
}

impl RightDelim {
    pub fn has_trim_marker(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TrimmedRightDelim
    }
}

define_token! {
    Ident(SyntaxKind::Ident)
}

impl Ident {
    pub fn get(&self) -> &str {
        self.syntax.text()
    }
}

define_token! {
    Field(SyntaxKind::Field)
}

impl Field {
    pub fn name(&self) -> Option<&str> {
        self.syntax.text().strip_prefix('.').filter(|name| !name.is_empty())
    }
}

define_token! {
    Var(SyntaxKind::Var)
}

impl Var {
    pub fn name(&self) -> &str {
        self.syntax.text()
    }
}

define_token! {
    Bool(SyntaxKind::Bool)
}

impl Bool {
    pub fn get(&self) -> bool {
        self.syntax.text() == "true"
    }
}

define_token! {
    Int(SyntaxKind::Int)
}

impl Int {
    pub fn get(&self) -> Option<i64> {
        go_lit_syntax::parse_int(self.syntax.text()).ok()
    }
}

define_token! {
    Float(SyntaxKind::Float)
}

impl Float {
    pub fn get(&self) -> Option<f64> {
        go_lit_syntax::parse_float(self.syntax().text()).ok()
    }
}

define_token! {
    Char(SyntaxKind::Char)
}

impl Char {
    pub fn get(&self) -> Option<char> {
        let mut s = Scanner::new(self.syntax.text());
        s.eat_if('\'');
        match s.eat() {
            Some('\\') => {
                let Some(after_slash) = s.eat() else { return None };
                go_lit_syntax::scan_escape_sequence(&mut s, after_slash, EscapeContext::CharacterLiteral).ok()
            }
            Some(c) => Some(c),
            None => None,
        }
    }
}

define_token! {
    Nil(SyntaxKind::Nil)
}

#[derive(Debug, Clone, Hash)]
pub enum StringLiteral {
    Interpreted(InterpretedString),
    Raw(RawString),
}

impl AstElement for StringLiteral {
    fn cast(element: SyntaxElement) -> Option<Self> {
        match element.kind() {
            SyntaxKind::InterpretedString => InterpretedString::cast(element).map(Self::Interpreted),
            SyntaxKind::RawString => RawString::cast(element).map(Self::Raw),
            _ => None,
        }
    }
}

impl<'a> StringLiteral {
    pub fn syntax(&self) -> &SyntaxToken {
        match self {
            Self::Interpreted(v) => &v.syntax,
            Self::Raw(v) => &v.syntax,
        }
    }

    pub fn get(&'a self) -> Cow<'a, str> {
        match self {
            Self::Interpreted(v) => v.get(),
            Self::Raw(v) => Cow::Borrowed(v.get()),
        }
    }
}

define_token! {
    InterpretedString(SyntaxKind::InterpretedString)
}

impl<'a> InterpretedString {
    pub fn get(&'a self) -> Cow<'a, str> {
        let content = strip_quotes(self.syntax.text(), '"');
        go_lit_syntax::interpret_string_content(content)
    }
}

define_token! {
    RawString(SyntaxKind::RawString)
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
