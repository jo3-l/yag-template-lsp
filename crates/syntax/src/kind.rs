#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    /// A syntax error.
    Error,
    /// The end of the file.
    Eof,

    /// Literal text outside of an action.
    Text,

    /// A left action delimiter: `{{`.
    LeftDelim,
    /// A left action delimiter with trim marker: `{{- `.
    TrimmedLeftDelim,
    /// A right action delimiter: `}}`.
    RightDelim,
    /// A right action delimiter with trim marker: ` -}}`.
    TrimmedRightDelim,
    /// A comment: `/* ... */`
    Comment,
    /// Whitespace.
    Whitespace,

    /// The declaration operator: `:=`.
    ColonEq,
    /// The assignment operator: `=`.
    Eq,

    /// The `if` keyword.
    If,
    /// The `end` keyword.
    End,

    /// The top-level node.
    Root,
    /// A list of actions and text, possibly interspersed.
    ActionList,
    /// The `{{end}}` clause completing a conditional or loop compound action.
    EndClause,
    /// An if-else compound action: `{{if x}} y {{else}} z {{end}}`
    IfConditional,
    /// The `{{if x}}` clause within an if-else compound action.
    IfClause,
    /// An expression used as an action, e.g., `{{fn 1 2 3}}`.
    ExprAction,
    /// An identifier.
    Ident,
    /// A function call: `f x y z ...`.
    FuncCall,
    /// A variable: `$x`.
    Var,
    /// A variable declaration: `$x := y`.
    VarDecl,
    /// A variable assignment: `$x = y`.
    VarAssign,
    /// A boolean literal.
    Bool,
    /// An integer literal.
    Int,

    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    pub fn from(kind: u16) -> SyntaxKind {
        assert!(kind < SyntaxKind::__LAST as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(kind) }
    }

    pub fn try_from_ident(ident: &str) -> Option<SyntaxKind> {
        use SyntaxKind::*;
        Some(match ident {
            "if" => If,
            "end" => End,

            "true" | "false" => Bool,
            _ => return None,
        })
    }

    pub fn is_left_delim(self) -> bool {
        matches!(self, SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
    }

    pub fn is_right_delim(self) -> bool {
        matches!(self, SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
    }

    pub fn is_trivia(self) -> bool {
        // NOTE: there are some productions in the grammar where whitespace is
        // significant but it is easier to treat these as exceptional cases
        // rather than the norm
        matches!(self, SyntaxKind::Comment | SyntaxKind::Whitespace)
    }

    pub fn name(self) -> &'static str {
        use SyntaxKind::*;
        match self {
            Error => "syntax error",
            Eof => "end of file",
            Text => "text",
            LeftDelim => "`{{`",
            TrimmedLeftDelim => "`{{- `",
            RightDelim => "`}}`",
            TrimmedRightDelim => "` -}}`",
            Comment => "comment",
            Whitespace => "whitespace",
            ColonEq => "`:=`",
            Eq => "`=`",
            If => "`if`",
            End => "`end`",
            Root => "root",
            ActionList => "action list",
            EndClause => "end clause",
            IfConditional => "if conditional",
            IfClause => "if clause",
            ExprAction => "action",
            Ident => "identifier",
            FuncCall => "function call",
            Bool => "boolean literal",
            Int => "integer literal",
            Var => "variable",
            VarAssign => "variable assignment",
            VarDecl => "variable declaration",

            __LAST => "",
        }
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(value: SyntaxKind) -> Self {
        rowan::SyntaxKind(value as u16)
    }
}
