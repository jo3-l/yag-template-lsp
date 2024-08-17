use std::collections::BTreeMap;

use anyhow::anyhow;
use tower_lsp::lsp_types::{Position, Range};
use yag_template_analysis::{self, Analysis};
use yag_template_syntax::ast::{self, SyntaxNodeExt};
use yag_template_syntax::parser::{self, Parse};
use yag_template_syntax::{SyntaxNode, TextRange, TextSize};

pub(crate) struct Document {
    pub(crate) parse: Parse,
    pub(crate) mapper: Mapper,
    pub(crate) analysis: Analysis,
}

impl Document {
    pub fn new(src: &str) -> anyhow::Result<Self> {
        let parse = parser::parse(src);
        let root = SyntaxNode::new_root(parse.root.clone())
            .try_to::<ast::Root>()
            .ok_or_else(|| anyhow!("root node of parse tree should be an ast::Root"))?;
        let document = Self {
            parse: parse.clone(),
            mapper: Mapper::new_utf16(src),
            analysis: yag_template_analysis::analyze(root),
        };
        Ok(document)
    }

    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.parse.root.clone())
    }
}

/// A mapper that translates offset:length bytes to 0-based line:row characters.
/// Modified from the `lsp-async-stub` crate (MIT license, Ferenc Tamás).
pub(crate) struct Mapper {
    byte_offset_to_pos: BTreeMap<TextSize, Position>,
    pos_to_byte_offset: BTreeMap<Position, TextSize>,
}

impl Mapper {
    pub(crate) fn new_utf16(src: &str) -> Self {
        let mut byte_offset_to_pos = BTreeMap::new();
        let mut pos_to_byte_offset = BTreeMap::new();

        let mut line = 0u32;
        let mut character = 0u32; // UTF-16 line length

        let mut cur_utf8_offset = 0u32;
        for c in src.chars() {
            let len_utf8 = c.len_utf8() as u32;
            byte_offset_to_pos.extend(
                (cur_utf8_offset..cur_utf8_offset + len_utf8)
                    .map(|b| (TextSize::from(b), Position { line, character })),
            );
            pos_to_byte_offset.insert(Position { line, character }, TextSize::from(cur_utf8_offset));

            cur_utf8_offset += len_utf8;
            character += c.len_utf16() as u32;
            if c == '\n' {
                // LF is at the start of each line.
                line += 1;
                character = 0;
            }
        }

        // Imaginary EOF character.
        byte_offset_to_pos.insert(TextSize::from(cur_utf8_offset), Position { line, character });
        pos_to_byte_offset.insert(Position { line, character }, TextSize::from(cur_utf8_offset));

        Self {
            byte_offset_to_pos,
            pos_to_byte_offset,
        }
    }

    pub(crate) fn offset(&self, position: Position) -> TextSize {
        self.pos_to_byte_offset
            .get(&position)
            .copied()
            .expect("position should be valid")
    }

    pub(crate) fn text_range(&self, range: Range) -> TextRange {
        TextRange::new(self.offset(range.start), self.offset(range.end))
    }

    pub(crate) fn position(&self, offset: TextSize) -> Position {
        self.byte_offset_to_pos
            .get(&offset)
            .copied()
            .expect("offset should be valid")
    }

    pub(crate) fn range(&self, range: TextRange) -> Range {
        Range {
            start: self.position(range.start()),
            end: self.position(range.end()),
        }
    }
}
