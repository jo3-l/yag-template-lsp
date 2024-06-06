mod actions;
mod exprs;
mod token_sets;

use actions::text_or_action;

pub use crate::parser::Parse;
use crate::parser::Parser;
use crate::{NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode};

pub fn parse(input: &str) -> Parse {
    let mut p = Parser::new(input);
    let m = p.marker();
    while !p.done() {
        text_or_action(&mut p);
    }
    p.wrap(m, SyntaxKind::Root);
    p.finish()
}

fn print(indent: usize, element: SyntaxElement) {
    let kind = element.kind();
    print!("{:indent$}", "");
    match element {
        NodeOrToken::Node(node) => {
            println!("- {:?} {:?}", kind, node.text_range());
            for child in node.children_with_tokens() {
                print(indent + 2, child);
            }
        }

        NodeOrToken::Token(token) => {
            println!("- {:?} {:?} {:?}", token.text(), kind, token.text_range())
        }
    }
}

#[test]
fn demo_parse() {
    let text = r#"{{if 1}}
    true
    {{else if asdfasdf}}
    bcd
    {{else}}
    no
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
