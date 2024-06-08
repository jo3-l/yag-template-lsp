#![allow(unused, unused_imports)]
use crate::parser::parse;
use crate::{SyntaxElement, SyntaxNode};

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
    let text = r#"{{define "foobar"}}
    abcd
    {{end}}
    "#;
    let parsed = parse(text);
    let node = SyntaxNode::new_root(parsed.root.clone());
    print(0, node.into());
    println!("errors");
    for err in parsed.errors {
        println!("{:?}: {}", err.range, err.message);
    }
}
