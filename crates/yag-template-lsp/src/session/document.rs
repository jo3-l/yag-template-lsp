use std::collections::BTreeMap;

use anyhow::anyhow;
use tower_lsp::lsp_types::{Position, Range};
use yag_template_analysis::{self, Analysis};
use yag_template_syntax::ast::{self, SyntaxNodeExt};
use yag_template_syntax::parser::{self, Parse};
use yag_template_syntax::{SyntaxNode, TextRange, TextSize, YagTemplateLanguage};

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
/// Extracted from the `lsp-async-stub` crate (MIT license, Ferenc Tamás).
pub(crate) struct Mapper {
    offset_to_position: BTreeMap<TextSize, Position>,
    position_to_offset: BTreeMap<Position, TextSize>,
    lines: usize,
    end: Position,
}

impl Mapper {
    pub(crate) fn new_utf16(source: &str) -> Self {
        let mut offset_to_position = BTreeMap::new();
        let mut position_to_offset = BTreeMap::new();

        let mut line = 0u32;
        let mut character = 0u32;
        let mut last_offset = 0;

        for c in source.chars() {
            let new_offset = last_offset + c.len_utf8();

            let character_size = c.len_utf16();

            offset_to_position
                .extend((last_offset..new_offset).map(|b| (TextSize::from(b as u32), Position { line, character })));

            position_to_offset
                .extend((last_offset..new_offset).map(|b| (Position { line, character }, TextSize::new(b as u32))));

            last_offset = new_offset;

            character += character_size as u32;
            if c == '\n' {
                // LF is at the start of each line.
                line += 1;
                character = 0;
            }
        }

        // Last imaginary character.
        offset_to_position.insert(TextSize::from(last_offset as u32), Position { line, character });
        position_to_offset.insert(Position { line, character }, TextSize::from(last_offset as u32));

        Self {
            offset_to_position,
            position_to_offset,
            lines: line as usize,
            end: Position { line, character },
        }
    }

    pub(crate) fn offset(&self, position: Position) -> Option<TextSize> {
        self.position_to_offset.get(&position).copied()
    }

    pub(crate) fn text_range(&self, range: Range) -> Option<TextRange> {
        self.offset(range.start)
            .and_then(|start| self.offset(range.end).map(|end| TextRange::new(start, end)))
    }

    pub(crate) fn position(&self, offset: TextSize) -> Option<Position> {
        self.offset_to_position.get(&offset).copied()
    }

    pub(crate) fn range(&self, range: TextRange) -> Option<Range> {
        self.position(range.start())
            .and_then(|start| self.position(range.end()).map(|end| Range { start, end }))
    }

    pub(crate) fn mappings(&self) -> (&BTreeMap<TextSize, Position>, &BTreeMap<Position, TextSize>) {
        (&self.offset_to_position, &self.position_to_offset)
    }

    pub(crate) fn line_count(&self) -> usize {
        self.lines
    }

    pub(crate) fn all_range(&self) -> Range {
        Range {
            start: Position { line: 0, character: 0 },
            end: self.end,
        }
    }
}
