//! Transform documentation from .ydef files into Discord markdown.

use std::iter;

pub(crate) fn render(doc: &str) -> String {
    const DISCORD_TAB_WIDTH: usize = 2;

    let unwrapped = unwrap_markdown(doc);
    tabs_to_spaces(&unwrapped, DISCORD_TAB_WIDTH)
}

fn tabs_to_spaces(s: &str, tab_width: usize) -> String {
    let mut reindented = String::with_capacity(s.len());
    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            reindented.push('\n');
        }

        let leading_tabs = line.bytes().take_while(|&c| c == b'\t').count();
        reindented.extend(iter::repeat(' ').take(leading_tabs * tab_width));
        reindented.push_str(&line[leading_tabs..]);
    }
    reindented
}

fn unwrap_markdown(s: &str) -> String {
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

    let mut output = String::with_capacity(s.len());
    let out_verbatim = |output: &mut String, s: &str| {
        output.push('\n');
        output.push_str(s);
    };
    let out_with_sep = |output: &mut String, sep: char, s: &str| {
        if output.bytes().next_back().is_some_and(|c| c != b'\n') {
            output.push(sep);
        }
        output.push_str(s);
    };

    let mut in_codeblock = false;
    for line in dedented_lines {
        if heuristic::has_codefence(line) {
            in_codeblock = !in_codeblock;
            out_with_sep(&mut output, '\n', line);
        } else if in_codeblock {
            out_verbatim(&mut output, line);
        } else if line.is_empty() || heuristic::is_list_item(line) {
            out_with_sep(&mut output, '\n', line);
        } else {
            out_with_sep(&mut output, ' ', line);
        }
    }
    output
}

mod heuristic {
    use std::sync::LazyLock;

    use regex::Regex;

    pub(super) fn has_codefence(line: &str) -> bool {
        line.starts_with("```")
    }

    pub(super) fn is_list_item(line: &str) -> bool {
        static LIST_ITEM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\s*(?:- |\d+\. )"#).unwrap());
        LIST_ITEM_RE.is_match(line)
    }
}
