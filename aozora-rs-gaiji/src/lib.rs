mod menkuten;
mod parser;
mod shift_jis;
mod whole;

use std::borrow::Cow;
use std::{collections::HashMap, sync::LazyLock};

use rkyv::util::AlignedVec;

pub type GaijiMap = HashMap<String, String>;
pub type RevGaijiMap = HashMap<String, String>;

pub use parser::{hex, parse_tag, shift_jis, unicode, white0};
use winnow::Parser;

pub use crate::parser::{Gaiji, TagSet, location};
pub use crate::whole::whole_gaiji_to_char;
pub use crate::{parser::Unicode, shift_jis::JISCharactor};

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

pub fn gaiji_to_char(input: &str) -> Option<Cow<'static, str>> {
    let mut input = input;
    parse_tag(location)
        .verify_map(|t| t.char())
        .parse_next(&mut input)
        .ok()
}
