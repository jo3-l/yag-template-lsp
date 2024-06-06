use std::collections::BTreeMap;

use tower_lsp::lsp_types::{Position, Range};
use yag_template_syntax::{TextRange, TextSize};

/// A mapper that translates offset:length bytes to 0-based line:row characters.
/// Extracted from the `lsp-async-stub` crate (MIT license, Ferenc Tamás).
pub struct Mapper {
    offset_to_position: BTreeMap<TextSize, Position>,
    position_to_offset: BTreeMap<Position, TextSize>,
    lines: usize,
    end: Position,
}

impl Mapper {
    pub fn new_utf16(source: &str) -> Self {
        let mut offset_to_position = BTreeMap::new();
        let mut position_to_offset = BTreeMap::new();

        let mut line = 0u32;
        let mut character = 0u32;
        let mut last_offset = 0;

        for c in source.chars() {
            let new_offset = last_offset + c.len_utf8();

            let character_size = c.len_utf16();

            offset_to_position.extend(
                (last_offset..new_offset)
                    .map(|b| (TextSize::from(b as u32), Position { line, character })),
            );

            position_to_offset.extend(
                (last_offset..new_offset)
                    .map(|b| (Position { line, character }, TextSize::new(b as u32))),
            );

            last_offset = new_offset;

            character += character_size as u32;
            if c == '\n' {
                // LF is at the start of each line.
                line += 1;
                character = 0;
            }
        }

        // Last imaginary character.
        offset_to_position.insert(
            TextSize::from(last_offset as u32),
            Position { line, character },
        );
        position_to_offset.insert(
            Position { line, character },
            TextSize::from(last_offset as u32),
        );

        Self {
            offset_to_position,
            position_to_offset,
            lines: line as usize,
            end: Position { line, character },
        }
    }

    #[must_use]
    pub fn offset(&self, position: Position) -> Option<TextSize> {
        self.position_to_offset.get(&position).copied()
    }

    #[must_use]
    pub fn text_range(&self, range: Range) -> Option<TextRange> {
        self.offset(range.start)
            .and_then(|start| self.offset(range.end).map(|end| TextRange::new(start, end)))
    }

    #[must_use]
    pub fn position(&self, offset: TextSize) -> Option<Position> {
        self.offset_to_position.get(&offset).copied()
    }

    #[must_use]
    pub fn range(&self, range: TextRange) -> Option<Range> {
        self.position(range.start())
            .and_then(|start| self.position(range.end()).map(|end| Range { start, end }))
    }

    #[must_use]
    pub fn mappings(&self) -> (&BTreeMap<TextSize, Position>, &BTreeMap<Position, TextSize>) {
        (&self.offset_to_position, &self.position_to_offset)
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines
    }

    #[must_use]
    pub fn all_range(&self) -> Range {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: self.end,
        }
    }
}
