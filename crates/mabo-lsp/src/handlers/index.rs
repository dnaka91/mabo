use std::ops::Range;

use anyhow::{Context, Result};
use line_index::{LineCol, LineIndex, TextSize, WideEncoding, WideLineCol};
use lsp_types as lsp;

pub struct Index {
    index: LineIndex,
    encoding: Option<WideEncoding>,
}

impl Index {
    pub fn new(index: LineIndex, encoding: &lsp::PositionEncodingKind) -> Self {
        Self {
            index,
            encoding: match encoding.as_str() {
                "utf-8" => None,
                "utf-32" => Some(WideEncoding::Utf32),
                _ => Some(WideEncoding::Utf16),
            },
        }
    }

    pub fn update(&mut self, index: LineIndex) {
        self.index = index;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn get_range(&self, location: impl Into<Range<usize>>) -> Result<lsp::Range> {
        let range = location.into();

        let start = self.index.line_col(TextSize::new(range.start as u32));
        let end = self.index.line_col(TextSize::new(range.end as u32));

        if let Some(encoding) = self.encoding {
            let start = self
                .index
                .to_wide(encoding, start)
                .context("missing start position in wide encoding")?;

            let end = self
                .index
                .to_wide(encoding, end)
                .context("missing end position in wide encoding")?;

            return Ok(lsp::Range::new(
                lsp::Position::new(start.line, start.col),
                lsp::Position::new(end.line, end.col),
            ));
        }

        Ok(lsp::Range::new(
            lsp::Position::new(start.line, start.col),
            lsp::Position::new(end.line, end.col),
        ))
    }

    pub fn get_offset(&self, position: lsp::Position) -> Result<usize> {
        let line_col = if let Some(encoding) = self.encoding {
            self.index
                .to_utf8(
                    encoding,
                    WideLineCol {
                        line: position.line,
                        col: position.character,
                    },
                )
                .context("missing utf-16 position")?
        } else {
            LineCol {
                line: position.line,
                col: position.character,
            }
        };

        Ok(self
            .index
            .offset(line_col)
            .context("missing offset position")?
            .into())
    }
}
