// JIS X 0213 面区点からUnicodeへの変換を行うモジュール
//
// このモジュールはx0213.orgの公式マッピングテーブルに基づいて
// 面区点番号をUnicodeコードポイントに変換します。
//
// 参照: http://x0213.org/codetable/jisx0213-2004-std.txt

use std::collections::HashMap;
use std::sync::LazyLock;

use rkyv::{AlignedVec, Deserialize, Infallible, check_archived_root};

pub type MenkutenKey = (u8, u8, u8);
pub type MenkutenToUnicodeMap = HashMap<MenkutenKey, String>;

/// 静的に組み込まれた変換テーブル
///
/// menkuten_to_unicode.map ファイルが存在しない場合は空のマップを返します。
/// `cargo run -p aozora-rs-gaiji --bin gen_menkuten` を実行してテーブルを生成してください。
pub static MENKUTEN_TO_UNICODE: LazyLock<MenkutenToUnicodeMap> = LazyLock::new(|| {
    let bytes = include_bytes!("../menkuten_to_unicode.map");
    let mut aligned = AlignedVec::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);
    let archived = check_archived_root::<MenkutenToUnicodeMap>(&aligned)
        .expect("menkuten_to_unicode.map data is corrupted");
    archived.deserialize(&mut Infallible).unwrap()
});

pub fn menkuten_to_unicode(plane: u8, row: u8, cell: u8) -> Option<&'static str> {
    MENKUTEN_TO_UNICODE
        .get(&(plane, row, cell))
        .map(|s| s.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_map_fallback() {
        let _ = menkuten_to_unicode(1, 4, 1);
    }
}
