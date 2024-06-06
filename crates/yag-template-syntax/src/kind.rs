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
    /// The pipe operator: `|`.
    Pipe,
    /// A left parenthesis: `(`.
    LeftParen,
    /// A right parenthesis: `)`.
    RightParen,

    /// A variable: `$x`.
    Var,
    /// An identifier.
    Ident,

    /// The `if` keyword.
    If,
    /// The `else` keyword.
    Else,
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
    /// A single `{{else}} actions...` branch of an if-else compound action.
    ElseBranch,
    /// The `{{else}}` or `{{else if x}}` clause within an else branch.
    ElseClause,
    /// An expression used as an action, e.g., `{{fn 1 2 3}}`.
    ExprAction,
    /// A function call: `f x y z ...`.
    FuncCall,
    /// An expression called with arguments: `.Foo.Bar x y z ...`. At the time
    /// of writing, the real template executor written in Go only supports
    /// calling methods with arguments, but for maximal generality we permit the
    /// callee to be any expression.
    ExprCall,
    /// A parenthesized expression: `(...)`.
    ParenthesizedExpr,
    /// A pipeline: `{{x | f y z | g a b c}}`.
    Pipeline,
    /// A single stage in a pipeline, comprising the pipe symbol and the
    /// function call.
    ///
    /// For instance, in the previous snippet `{{x | f y z | g a b c}}`, there
    /// are two stages, `| f y z` and `| g a b c`.
    PipelineStage,
    /// A variable that is evaluated as an expression (not as part of a
    /// declaration of an assignment): `$x`.
    VarAccess,
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
            "else" => Else,
            "end" => End,

            "true" | "false" => Bool,
            _ => return None,
        })
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
            Pipe => "`|`",
            LeftParen => "`(`",
            RightParen => "`)`",
            Ident => "identifier",
            Var => "variable",
            If => "`if`",
            Else => "`else`",
            End => "`end`",
            Root => "root",
            ActionList => "action list",
            EndClause => "end clause",
            IfConditional => "if conditional",
            IfClause => "if clause",
            ElseBranch => "else branch",
            ElseClause => "else clause",
            ExprAction => "action",
            FuncCall => "function call",
            ExprCall => "expression called with arguments",
            ParenthesizedExpr => "parenthesized expression",
            Pipeline => "pipeline",
            PipelineStage => "pipeline stage",
            Bool => "boolean literal",
            Int => "integer literal",
            VarAccess => "variable access",
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
