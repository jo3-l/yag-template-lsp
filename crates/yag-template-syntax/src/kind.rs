use core::fmt;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    /// The end of the file.
    Eof,
    /// An invalid character in an action.
    InvalidCharInAction,

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
    /// Whitespace in an action.
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

    /// A boolean literal.
    Bool,
    /// An integer literal.
    Int,
    /// A floating-point literal.
    Float,
    /// An interpreted (i.e., double-quoted) string literal: `"..."`.
    InterpretedString,
    /// A raw string literal: `` `...` ``.
    RawString,
    /// A character literal: `'c'`.
    Char,
    /// An untyped nil constant: `nil`.
    Nil,

    /// The `if` keyword.
    If,
    /// The `with` keyword.
    With,
    /// The `else` keyword.
    Else,
    /// The `end` keyword.
    End,
    /// The `range` keyword.
    Range,
    /// The `while` keyword.
    While,
    /// The `try` keyword.
    Try,
    /// The `catch` keyword.
    Catch,
    /// The `define` keyword.
    Define,
    /// The `template` keyword.
    Template,
    /// The `block` keyword.
    Block,
    /// The `break` keyword.
    Break,
    /// The `continue` keyword.
    Continue,
    /// The `return` keyword.
    Return,

    // SyntaxKinds past this point correspond to nodes generated by the parser
    // (whereas previous kinds are tokens generated by the lexer.)
    #[doc(hidden)]
    __LAST_TOKEN_KIND,

    /// An error node containing syntactically invalid code.
    Error,
    /// The top-level node.
    Root,
    /// A list of actions and text, possibly interspersed.
    ActionList,
    /// An action that only contains a comment: `{{/* comment */}}`.
    CommentAction,
    /// An associated template definition: `{{define "name"}} ... {{end}}`
    TemplateDefinition,
    /// The `{{define "name"}}` clause in an associated template definition.
    DefineClause,
    /// An immediately invoked associated template block: `{{block "name" expr}} ... {{end}}`.
    TemplateBlock,
    /// The `{{block "name" expr}}` clause in an immediately invoked associated template block.
    BlockClause,
    /// An associated template invocation: `{{template "name" expr}}`.
    TemplateInvocation,
    /// The `{{return expr}}` action.
    ReturnAction,
    /// The `{{end}}` clause completing a conditional or loop compound action.
    EndClause,
    /// An if-else compound action: `{{if x}} ... {{else if y}} ... {{else}} ... {{end}}`
    IfAction,
    /// The `{{if x}}` clause within an if-else compound action.
    IfClause,
    /// A single `{{else}} actions...` branch of an if-else or with-else compound action.
    ElseBranch,
    /// The `{{else}}` or `{{else if x}}` clause within an else branch.
    ElseClause,
    /// A with-else compound action: `{{with x}} ... {{else if y}} ... {{else}} ... {{end}}`.
    WithAction,
    /// The `{{with x}}` clause within a with-else compound action.
    WithClause,
    /// A range loop compound action: `{{range x}} ... {{else}} ... {{end}}`.
    RangeLoop,
    /// The `{{range ...}}` clause within a range loop compound action.
    RangeClause,
    /// A while loop compound action: `{{while x}} ... {{else}} ... {{end}}`.
    WhileLoop,
    /// The `{{while ...}}` clause within a while loop compound action.
    WhileClause,
    /// The `{{break}}` action.
    LoopBreak,
    /// The `{{continue}}` action.
    LoopContinue,
    /// A try-catch compound action: `{{try}} ... {{catch}} ... {{end}}`.
    TryCatchAction,
    /// The `{{try}}` clause within a try-catch compound action.
    TryClause,
    /// The `{{catch}}` clause within a try-catch compound action.
    CatchClause,
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
    /// The parser will produce a `ContextFieldChain` even in the case where there
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
    /// A literal constant.
    Literal,

    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    pub fn from(kind: u16) -> SyntaxKind {
        assert!(kind < SyntaxKind::__LAST as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(kind) }
    }

    pub fn from_ident(ident: &str) -> Option<SyntaxKind> {
        use SyntaxKind::*;
        Some(match ident {
            "if" => If,
            "with" => With,
            "else" => Else,
            "end" => End,
            "range" => Range,
            "while" => While,
            "try" => Try,
            "catch" => Catch,
            "define" => Define,
            "template" => Template,
            "block" => Block,
            "break" => Break,
            "continue" => Continue,
            "return" => Return,

            "nil" => Nil,
            "true" | "false" => Bool,
            _ => return None,
        })
    }

    pub fn is_trivia(self) -> bool {
        // NOTE: Whitespace is significant in some parts of the grammar, so is not considered
        // trivia.
        self == SyntaxKind::Comment
    }

    pub fn is_literal(self) -> bool {
        use SyntaxKind::*;
        matches!(self, Bool | Int | Float | InterpretedString | RawString | Char | Nil)
    }
}

impl fmt::Display for SyntaxKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SyntaxKind::*;
        f.write_str(match self {
            Eof => "end of file",
            InvalidCharInAction => "invalid character in action",

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
            Bool => "boolean",
            Int => "integer",
            Float => "float",
            InterpretedString => "double-quoted string",
            RawString => "raw string",
            Char => "character literal",
            Nil => "nil constant",

            If => "`if`",
            With => "`with`",
            Else => "`else`",
            End => "`end`",
            Range => "`range`",
            While => "`while`",
            Try => "`try`",
            Catch => "`catch`",
            Define => "`define`",
            Template => "`template`",
            Block => "`block`",
            Break => "`break`",
            Continue => "`continue`",
            Return => "`return`",

            __LAST_TOKEN_KIND => "<SyntaxKind::__LAST_TOKEN_KIND>",

            Error => "syntax error",
            Root => "root",
            ActionList => "action list",
            CommentAction => "comment action",
            TemplateDefinition => "template definition",
            DefineClause => "`define` clause",
            TemplateBlock => "template block",
            BlockClause => "`block` clause",
            TemplateInvocation => "template invocation",
            ReturnAction => "return action",
            EndClause => "end clause",
            IfAction => "if conditional",
            IfClause => "if clause",
            ElseBranch => "else branch",
            ElseClause => "else clause",
            WithAction => "with conditional",
            WithClause => "with clause",
            RangeLoop => "range loop",
            RangeClause => "range clause",
            WhileLoop => "while loop",
            WhileClause => "while clause",
            LoopBreak => "break action",
            LoopContinue => "continue action",
            TryCatchAction => "try-catch action",
            TryClause => "try clause",
            CatchClause => "catch clause",
            ExprAction => "expression in action context",

            FuncCall => "function call",
            ExprCall => "expression called with arguments",
            ParenthesizedExpr => "parenthesized expression",
            Pipeline => "pipeline",
            PipelineStage => "pipeline stage",
            ContextAccess => "context access",
            ContextFieldChain => "context field chain",
            ExprFieldChain => "expression field chain",
            VarAccess => "variable access",
            VarAssign => "variable assignment",
            VarDecl => "variable declaration",
            Literal => "literal constant",

            __LAST => "<SyntaxKind::__LAST>",
        })
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(value: SyntaxKind) -> Self {
        rowan::SyntaxKind(value as u16)
    }
}
