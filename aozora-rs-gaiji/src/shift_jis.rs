fn shift_char(bytes: &[u8; 2]) -> Option<char> {
    encoding_rs::SHIFT_JIS.decode(bytes).0.chars().next()
}

pub fn get_shift_jis_char(face: u16, area: u16, point: u16) -> Option<char> {
    if !(1..=94).contains(&area) || !(1..=94).contains(&point) {
        return None;
    }

    let (s1, s2) = match face {
        1 => {
            let offset = if area <= 62 { 0x81 } else { 0xC1 };
            let s1 = ((area - 1) >> 1) + offset;

            let s2 = if area % 2 == 1 {
                let mut code = point + 0x3F;
                if code >= 0x7F {
                    code += 1;
                }
                code
            } else {
                point + 0x9E
            };

            (s1, s2)
        }
        2 => {
            let s1 = if [1, 3, 4, 5, 8, 12, 13, 14, 15].contains(&area) {
                ((area + 0x1DF) >> 1) - (if area >= 8 { 1 } else { 0 }) * 3
            } else if (78..=94).contains(&area) {
                (area + 0x19C) >> 1
            } else {
                return None;
            };

            let s2 = if area % 2 == 1 {
                let mut code = point + 0x3F;
                if code >= 0x7F {
                    code += 1;
                }
                code
            } else {
                point + 0x9E
            };

            (s1, s2)
        }
        _ => return None,
    };

    if s1 > 0xFF || s2 > 0xFF {
        return None;
    }
    shift_char(&[s1 as u8, s2 as u8])
}

#[derive(Debug, Clone, Copy)]
pub struct JISCharactor {
    level: u16,
    face: u16,
    area: u16,
    point: u16,
}

impl JISCharactor {
    pub fn new(level: u16, face: u16, area: u16, point: u16) -> Option<Self> {
        let valid_area_and_point = 1..=94;
        let is_valid = [0, 1, 2].contains(&face)
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

    pub fn to_char(&self) -> Option<char> {
        get_shift_jis_char(self.face, self.area, self.point)
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
