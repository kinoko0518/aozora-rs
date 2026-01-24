// JIS X 0213 面区点番号から文字への変換
//
// このモジュールは面区点→Unicode変換テーブルを使用して
// JIS文字コードをUnicode文字に変換します。

use crate::menkuten::menkuten_to_unicode;

/// JIS X 0213 の面区点番号を表す構造体
#[derive(Debug, Clone, Copy)]
pub struct JISCharactor {
    level: u16, // 水準（1-4）
    face: u16,  // 面（1 or 2）
    area: u16,  // 区（1-94）
    point: u16, // 点（1-94）
}

impl JISCharactor {
    /// 新しいJISCharactorを作成
    ///
    /// # 引数
    /// * `level` - 水準（1-4）
    /// * `face` - 面（1 or 2）
    /// * `area` - 区（1-94）
    /// * `point` - 点（1-94）
    pub fn new(level: u16, face: u16, area: u16, point: u16) -> Option<Self> {
        let valid_area_and_point = 1..=94;
        let is_valid = [1, 2].contains(&face)
            && valid_area_and_point.contains(&area)
            && valid_area_and_point.contains(&point);
        if !is_valid {
            return None;
        }
        Some(Self {
            level,
            face,
            area,
            point,
        })
    }

    /// Unicode文字列に変換
    ///
    /// 面区点→Unicode変換テーブルを使用して変換します。
    pub fn to_char(&self) -> Option<&'static str> {
        menkuten_to_unicode(self.face as u8, self.area as u8, self.point as u8)
    }
}

impl std::fmt::Display for JISCharactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "第{}水準{}-{}-{}",
            self.level, self.face, self.area, self.point
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_jis_character() {
        let jis = JISCharactor::new(3, 1, 84, 77);
        assert!(jis.is_some());
        let jis = jis.unwrap();
        assert_eq!(jis.level, 3);
        assert_eq!(jis.face, 1);
        assert_eq!(jis.area, 84);
        assert_eq!(jis.point, 77);
    }

    #[test]
    fn test_invalid_face() {
        let jis = JISCharactor::new(3, 0, 84, 77);
        assert!(jis.is_none());

        let jis = JISCharactor::new(3, 3, 84, 77);
        assert!(jis.is_none());
    }

    #[test]
    fn test_invalid_area() {
        let jis = JISCharactor::new(3, 1, 0, 77);
        assert!(jis.is_none());

        let jis = JISCharactor::new(3, 1, 95, 77);
        assert!(jis.is_none());
    }

    #[test]
    fn test_display() {
        let jis = JISCharactor::new(3, 1, 84, 77).unwrap();
        assert_eq!(format!("{}", jis), "第3水準1-84-77");
    }
}
