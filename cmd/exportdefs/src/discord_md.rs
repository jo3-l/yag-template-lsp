//! Transform documentation from .ydef files into Discord markdown.

use std::iter;
use std::sync::LazyLock;

use regex::Regex;

pub(crate) fn render(doc: &str) -> String {
    const DISCORD_TAB_WIDTH: usize = 2;

    let contents = interpret_markdown(doc);
    let contents = reindent_with_spaces(contents, DISCORD_TAB_WIDTH);
    tight_layout(contents)
}

#[derive(Debug, Clone)]
enum Content {
    /// A paragraph.
    Paragraph(String),
    /// A line that is part of a codeblock.
    CodeItem(String),
    /// A line that is part of a list.
    ListItem(String),
    /// A heading.
    Heading(String),
    /// An empty line.
    Hardbreak,
}

impl Content {
    fn is_paragraph(&self) -> bool {
        matches!(self, Content::Paragraph(_))
    }
}

fn tight_layout(content: Vec<Content>) -> String {
    let mut buf = String::new();
    let emit_line = |buf: &mut String, s: &str| {
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(s);
    };

    let mut iter = content.into_iter().peekable();
    let mut prev_was_paragraph = false;
    while let Some(item) = iter.next() {
        let is_paragraph = item.is_paragraph();
        match item {
            Content::Paragraph(para) => emit_line(&mut buf, &para),
            Content::CodeItem(line) => emit_line(&mut buf, &line),
            Content::ListItem(line) => emit_line(&mut buf, &line),
            Content::Heading(line) => emit_line(&mut buf, &line),
            Content::Hardbreak => {
                if prev_was_paragraph && iter.peek().is_some_and(Content::is_paragraph) {
                    // Only keep blank lines when they delimit adjacent paragraphs.
                    buf.push('\n');
                }
            }
        }

        prev_was_paragraph = is_paragraph;
    }
    buf
}

fn interpret_markdown(s: &str) -> Vec<Content> {
    let common_indent = s
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let without_indent = line.trim_start();
            line.len() - without_indent.len()
        })
        .min()
        .unwrap_or(0);

    let dedented_lines = s
        .lines()
        .map(|line| if line.is_empty() { line } else { &line[common_indent..] });

    let mut out: Vec<Content> = Vec::new();
    let mut in_codeblock = false;
    for line in dedented_lines {
        let codefence = is_codefence(line);
        if codefence {
            in_codeblock = !in_codeblock;
        }

        if codefence || in_codeblock {
            out.push(Content::CodeItem(line.into()));
        } else if is_list_item(line) {
            out.push(Content::ListItem(line.into()));
        } else if is_heading(line) {
            out.push(Content::Heading(line.into()))
        } else if line.is_empty() {
            out.push(Content::Hardbreak);
        } else {
            match out.last_mut() {
                Some(Content::Paragraph(para)) => {
                    para.push(' ');
                    para.push_str(line);
                }
                _ => out.push(Content::Paragraph(line.into())),
            }
        }
    }
    out
}

fn is_codefence(line: &str) -> bool {
    line.starts_with("```")
}

fn is_list_item(line: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\s*(?:- |\d+\. )"#).unwrap());
    RE.is_match(line)
}

fn is_heading(line: &str) -> bool {
    line.starts_with("#")
}

fn reindent_with_spaces(contents: Vec<Content>, tab_width: usize) -> Vec<Content> {
    contents
        .into_iter()
        .map(|item| match item {
            Content::Paragraph(para) => Content::Paragraph(reindent_line(&para, tab_width)),
            Content::CodeItem(line) => Content::CodeItem(reindent_line(&line, tab_width)),
            Content::ListItem(line) => Content::ListItem(reindent_line(&line, tab_width)),
            Content::Heading(line) => Content::Heading(reindent_line(&line, tab_width)),
            Content::Hardbreak => Content::Hardbreak,
        })
        .collect()
}

fn reindent_line(line: &str, tab_width: usize) -> String {
    let leading_tabs = line.bytes().take_while(|&c| c == b'\t').count();
    let mut buf = String::with_capacity(line.len() + leading_tabs * tab_width - leading_tabs);
    buf.extend(iter::repeat(' ').take(leading_tabs * tab_width));
    buf.push_str(&line[leading_tabs..]);
    buf
}
