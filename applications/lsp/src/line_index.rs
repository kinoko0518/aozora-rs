use tower_lsp::lsp_types::Position;

/// バイトオフセットとLSP Position（行番号＋UTF-16文字オフセット）を相互変換する。
pub struct LineIndex {
    /// 各行の開始バイトオフセット
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub fn offset_to_position(&self, text: &str, offset: usize) -> Position {
        let offset = offset.min(text.len());
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line];
        let line_text = &text[line_start..offset];
        // LSPはUTF-16オフセットを要求する
        let character = line_text.encode_utf16().count() as u32;
        Position {
            line: line as u32,
            character,
        }
    }



    /// LSP PositionからバイトオフセットへXmlの変換
    pub fn position_to_offset(&self, text: &str, pos: Position) -> usize {
        let line = pos.line as usize;
        if line >= self.line_starts.len() {
            return text.len();
        }
        let line_start = self.line_starts[line];
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(text.len());
        let line_text = &text[line_start..line_end];
        // UTF-16オフセットからバイトオフセットに変換
        let mut utf16_count = 0u32;
        let mut byte_offset = 0;
        for ch in line_text.chars() {
            if utf16_count >= pos.character {
                break;
            }
            utf16_count += ch.len_utf16() as u32;
            byte_offset += ch.len_utf8();
        }
        line_start + byte_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_positions() {
        let text = "hello\nworld\n";
        let idx = LineIndex::new(text);
        assert_eq!(
            idx.offset_to_position(text, 0),
            Position { line: 0, character: 0 }
        );
        assert_eq!(
            idx.offset_to_position(text, 5),
            Position { line: 0, character: 5 }
        );
        // 'w' = line 1, char 0
        assert_eq!(
            idx.offset_to_position(text, 6),
            Position { line: 1, character: 0 }
        );
    }

    #[test]
    fn japanese_positions() {
        let text = "春と修羅\n宮沢賢治\n";
        let idx = LineIndex::new(text);
        // '春' = 3 bytes, UTF-16 offset = 0
        assert_eq!(
            idx.offset_to_position(text, 0),
            Position { line: 0, character: 0 }
        );
        // 'と' は offset 3 (バイト), UTF-16 offset 1
        assert_eq!(
            idx.offset_to_position(text, 3),
            Position { line: 0, character: 1 }
        );
        // 改行の次 = 行1
        let line1_start = "春と修羅\n".len();
        assert_eq!(
            idx.offset_to_position(text, line1_start),
            Position { line: 1, character: 0 }
        );
    }

    #[test]
    fn roundtrip() {
        let text = "［＃太字］テスト［＃太字終わり］\n";
        let idx = LineIndex::new(text);
        let offset = 6; // '＃' の先頭
        let pos = idx.offset_to_position(text, offset);
        let back = idx.position_to_offset(text, pos);
        assert_eq!(offset, back);
    }
}
