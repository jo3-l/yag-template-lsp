use std::borrow::Cow;

use unscanny::Scanner;

use crate::ast::AstToken;
use crate::go_lit_syntax::EscapeContext;
use crate::{go_lit_syntax, SyntaxKind, SyntaxToken};

macro_rules! define_token {
    ($(#[$attr:meta])* $name:ident($pat:pat)) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name {
            pub(crate) syntax: SyntaxToken,
        }

        impl AstToken for $name {
            fn cast(syntax: SyntaxToken) -> Option<Self> {
                if matches!(syntax.kind(), $pat) {
                    Some(Self { syntax })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxToken {
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
        go_lit_syntax::parse_float(self.syntax.text()).ok()
    }
}

define_token! {
    Rune(SyntaxKind::Rune)
}

impl Rune {
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StringLiteral {
    Interpreted(InterpretedString),
    Raw(RawString),
}

impl StringLiteral {
    pub fn get(&self) -> Cow<str> {
        match self {
            StringLiteral::Interpreted(v) => v.get(),
            StringLiteral::Raw(v) => Cow::Borrowed(v.get()),
        }
    }
}

impl AstToken for StringLiteral {
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

define_token! {
    InterpretedString(SyntaxKind::InterpretedString)
}

impl InterpretedString {
    pub fn get(&self) -> Cow<str> {
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
