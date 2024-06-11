#[allow(unused)]
use crate::ast::{self, AstNode, SyntaxNodeExt};
#[allow(unused)]
use crate::{parser, SyntaxElement, SyntaxNode};

fn print(indent: usize, element: SyntaxElement) {
    let kind = element.kind();
    print!("{:indent$}", "");
    match element {
        SyntaxElement::Node(node) => {
            println!("- {:?} {:?}", kind, node.text_range());
            for child in node.children_with_tokens() {
                print(indent + 2, child);
            }
        }

        SyntaxElement::Token(token) => {
            println!("- {:?} {:?} {:?}", token.text(), kind, token.text_range())
        }
    }
}

#[test]
fn demo_parse() {
    let src = r#"
{{$t := "test"}}
{{if 1}}
    {{$x := "test"}}
    {{$y := "foo"}}
    {{$bcd := "test"}}
{{end}}
{{template "foo" $z := 123}}
{{range $x, $y := "test" }}
    {{$"#;
    let parse = parser::parse(src);
    let node = SyntaxNode::new_root(parse.root.clone());
    print(0, node.clone().into());
    if !parse.errors.is_empty() {
        println!();
        println!("{} errors", parse.errors.len());
        for err in parse.errors {
            println!("{:?}: {}", err.range, err.message);
        }
    }

    println!();
    println!("ast: root");
    let root = node.to::<ast::Root>();
    for action in root.actions() {
        println!("  - {}", action.syntax().kind())
    }
}
