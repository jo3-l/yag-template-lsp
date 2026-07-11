//! Indexed logical-line offsets for source-oriented formatter passes.

/// Byte offsets for every logical source line. Construct this once when a pass
/// needs to map syntax ranges to lines, rather than repeatedly scanning source
/// prefixes for newline characters.
#[derive(Debug)]
pub(super) struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub(super) fn new(source: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(offset, byte)| (byte == b'\n').then_some(offset + 1)),
        );
        Self { starts }
    }

    pub(super) fn len(&self) -> usize {
        self.starts.len()
    }

    pub(super) fn line_for(&self, offset: usize) -> usize {
        self.starts.partition_point(|start| *start <= offset).saturating_sub(1)
    }

    #[allow(dead_code)] // Used by later lowering when it splits direct text by line.
    pub(super) fn start_of(&self, line: usize) -> usize {
        self.starts[line]
    }

    #[allow(dead_code)] // Used by later lowering when it splits direct text by line.
    pub(super) fn end_of(&self, line: usize, source_len: usize) -> usize {
        self.starts.get(line + 1).copied().unwrap_or(source_len)
    }
}
