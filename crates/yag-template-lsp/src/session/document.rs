use std::collections::BTreeMap;

use anyhow::Context;
use rowan::{TextRange, TextSize};
use tower_lsp::lsp_types::{Location, Position, Range, Url};
use yag_template_analysis::Analysis;
use yag_template_syntax::ast::SyntaxNodeExt;
use yag_template_syntax::parser::Parse;
use yag_template_syntax::query::Query;
use yag_template_syntax::{ast, parser, SyntaxNode};

use super::Session;

pub(crate) struct Document {
    pub(crate) uri: Url,
    pub(crate) parse: Parse,
    pub(crate) mapper: Mapper,
    pub(crate) analysis: Analysis,
}

impl Document {
    pub(crate) fn new(sess: &Session, uri: Url, src: &str) -> anyhow::Result<Self> {
        let parse = parser::parse(src);
        let root = SyntaxNode::new_root(parse.root.clone()).to::<ast::Root>();
        let document = Self {
            uri,
            parse: parse.clone(),
            mapper: Mapper::new(src),
            analysis: yag_template_analysis::analyze(&sess.envdefs, root),
        };
        Ok(document)
    }

    pub(crate) fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.parse.root.clone())
    }

    pub(crate) fn query_at(&self, pos: Position) -> anyhow::Result<Query> {
        Query::at(&self.syntax(), self.mapper.offset(pos)).context("failed querying syntax tree")
    }

    pub(crate) fn location_for(&self, range: TextRange) -> Location {
        Location::new(self.uri.clone(), self.mapper.range(range))
    }
}

/// A mapper that translates offset:length bytes to 0-based line:row characters.
/// Modified from the `lsp-async-stub` crate (MIT license, Ferenc Tamás).
pub(crate) struct Mapper {
    byte_offset_to_pos: BTreeMap<TextSize, Position>,
    pos_to_byte_offset: BTreeMap<Position, TextSize>,
}

impl Mapper {
    pub(crate) fn new(src: &str) -> Self {
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
