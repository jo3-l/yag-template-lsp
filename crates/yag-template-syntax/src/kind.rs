#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    /// The end of the file.
    Eof,
    /// A syntax error.
    Error,
    /// An invalid character in an action.
    InvalidChar,

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

    /// The comma separating iteration variables in range clauses: `,`.
    Comma,
    /// The declaration operator: `:=`.
    ColonEq,
    /// The assignment operator: `=`.
    Eq,
    /// The pipe operator: `|`.
    Pipe,
    /// The context or field access operator: `.`.
    Dot,
    /// A left parenthesis: `(`.
    LeftParen,
    /// A right parenthesis: `)`.
    RightParen,

    /// A variable: `$x`.
    Var,
    /// An identifier.
    Ident,
    /// A single field access, part of a context or expression field chain:
    /// `.Field`.
    Field,

    /// The `if` keyword.
    If,
    /// The `else` keyword.
    Else,
    /// The `end` keyword.
    End,
    /// The `range` keyword.
    Range,
    /// The `while` keyword.
    While,

    /// The top-level node.
    Root,
    /// A list of actions and text, possibly interspersed.
    ActionList,
    /// The `{{end}}` clause completing a conditional or loop compound action.
    EndClause,
    /// An if-else compound action: `{{if x}} ... {{else if y}} ... {{else}} ... {{end}}`
    IfConditional,
    /// The `{{if x}}` clause within an if-else compound action.
    IfClause,
    /// A single `{{else}} actions...` branch of an if-else compound action.
    ElseBranch,
    /// The `{{else}}` or `{{else if x}}` clause within an else branch.
    ElseClause,
    /// A range loop compound action: `{{range x}} ... {{else}} ... {{end}}`.
    RangeLoop,
    /// The `{{range ...}}` clause within a range loop compound action.
    RangeClause,
    /// A while loop compound action: `{{while x}} ... {{else}} ... {{end}}`.
    WhileLoop,
    /// The `{{while ...}}` clause within a while loop compound action.
    WhileClause,
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
    /// A single `.` that evaluates to the context data (not part of a field.)
    ContextAccess,
    /// A series of field accesses on the context data: `.Field1.Field2.Field3`.
    /// The parser will produce a ContextFieldChain even in the case where there
    /// is only one field.
    ContextFieldChain,
    /// A series of field accesses on an expression: `(...).Field1.Field2.Field3`.
    ExprFieldChain,
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
            "range" => Range,
            "while" => While,

            "true" | "false" => Bool,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        use SyntaxKind::*;
        match self {
            Eof => "end of file",
            Error => "syntax error",
            InvalidChar => "invalid character in action",
            Text => "text",
            LeftDelim => "`{{`",
            TrimmedLeftDelim => "`{{- `",
            RightDelim => "`}}`",
            TrimmedRightDelim => "` -}}`",
            Comment => "comment",
            Whitespace => "whitespace",
            Comma => "comma",
            ColonEq => "`:=`",
            Eq => "`=`",
            Pipe => "`|`",
            Dot => "`.`",
            LeftParen => "`(`",
            RightParen => "`)`",
            Ident => "identifier",
            Var => "variable",
            Field => "field",
            If => "`if`",
            Else => "`else`",
            End => "`end`",
            Range => "`range`",
            While => "`while`",
            Root => "root",
            ActionList => "action list",
            EndClause => "end clause",
            IfConditional => "if conditional",
            IfClause => "if clause",
            ElseBranch => "else branch",
            ElseClause => "else clause",
            RangeLoop => "range loop",
            RangeClause => "range clause",
            WhileLoop => "while loop",
            WhileClause => "while clause",
            ExprAction => "action",
            FuncCall => "function call",
            ExprCall => "expression called with arguments",
            ParenthesizedExpr => "parenthesized expression",
            Pipeline => "pipeline",
            PipelineStage => "pipeline stage",
            ContextAccess => "context access",
            ContextFieldChain => "context field chain",
            ExprFieldChain => "expression field chain",
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
