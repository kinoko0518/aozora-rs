mod whole;

use std::borrow::Cow;
use std::{collections::HashMap, sync::LazyLock};
use winnow::Parser;

pub use crate::whole::whole_gaiji_to_char;
pub use gaiji_chuki_parser::{GaijiChuki, parse_tag};

use rkyv::util::AlignedVec;

pub type GaijiMap = HashMap<String, String>;
pub type RevGaijiMap = HashMap<String, String>;

pub type MenkutenKey = (u8, u8, u8);
pub type MenkutenToUnicodeMap = HashMap<MenkutenKey, String>;

pub static GAIJI_TO_CHAR: LazyLock<GaijiMap> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/gaiji_to_char.map"));
    let mut aligned = AlignedVec::<16>::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);

    rkyv::from_bytes::<GaijiMap, rkyv::rancor::Error>(&aligned)
        .expect("gaiji_to_char.map data is corrupted")
});

pub static CHAR_TO_GAIJI: LazyLock<RevGaijiMap> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/char_to_gaiji.map"));
    let mut aligned = AlignedVec::<16>::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);

    rkyv::from_bytes::<RevGaijiMap, rkyv::rancor::Error>(&aligned)
        .expect("char_to_gaiji.map data is corrupted")
});

pub static MENKUTEN_TO_UNICODE: LazyLock<MenkutenToUnicodeMap> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/menkuten_to_unicode.map"));
    let mut aligned = AlignedVec::<16>::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);

    rkyv::from_bytes::<MenkutenToUnicodeMap, rkyv::rancor::Error>(&aligned)
        .expect("menkuten_to_unicode.map data is corrupted")
});

pub fn menkuten_to_unicode(plane: u8, row: u8, cell: u8) -> Option<&'static str> {
    MENKUTEN_TO_UNICODE
        .get(&(plane, row, cell))
        .map(|s| s.as_str())
}

pub fn gaiji_to_char(input: &mut &str) -> Option<Cow<'static, str>> {
    parse_tag
        .verify_map(|t| t.to_cow(&GAIJI_TO_CHAR, &MENKUTEN_TO_UNICODE))
        .parse_next(input)
        .ok()
}
