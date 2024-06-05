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
    /// An expression used as an action, e.g., `{{fn 1 2 3}}`.
    ExprAction,
    /// A function call: `f x y z ...`.
    FuncCall,
    /// An identifier.
    Ident,
    /// An if-else compound action: `{{if x}} y {{else}} z {{end}}`
    IfConditional,
    /// The `{{if x}}` clause within an if-else compound action.
    IfClause,
    /// An integer literal.
    Int,
    /// A variable: `$x`.
    Var,
    /// A variable declaration: `$x := y`.
    VarDecl,
    /// A variable assignment: `$x = y`.
    VarAssign,

    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    pub fn from(kind: u16) -> SyntaxKind {
        assert!(kind < SyntaxKind::__LAST as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(kind) }
    }

    pub fn try_from_keyword(keyword: &str) -> Option<SyntaxKind> {
        use SyntaxKind::*;
        Some(match keyword {
            "if" => If,
            "end" => End,
            _ => return None,
        })
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
            ActionList => "block",
            EndClause => "end clause",
            ExprAction => "action",
            FuncCall => "function call",
            Ident => "identifier",
            IfConditional => "if conditional",
            IfClause => "if clause",
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
