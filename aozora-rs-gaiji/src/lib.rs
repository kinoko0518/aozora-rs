mod parser;
mod shift_jis;

use std::{collections::HashMap, sync::LazyLock};

use rkyv::{AlignedVec, Deserialize, Infallible, check_archived_root};
use winnow::{Parser, error::ContextError};

pub type GaijiMap = HashMap<String, char>;
pub type RevGaijiMap = HashMap<char, String>;

pub use parser::{hex, parse_tag, shift_jis, unicode, white0};

pub use crate::parser::{Gaiji, TagSet, location};
pub use crate::{parser::Unicode, shift_jis::JISCharactor};

pub static GAIJI_TO_CHAR: LazyLock<GaijiMap> = LazyLock::new(|| {
    let bytes = include_bytes!("../gaiji_to_char.map");
    let mut aligned = AlignedVec::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);
    let archived =
        check_archived_root::<GaijiMap>(&aligned).expect("gaiji_to_char.map data is corrupted");
    archived.deserialize(&mut Infallible).unwrap()
});

pub static CHAR_TO_GAIJI: LazyLock<RevGaijiMap> = LazyLock::new(|| {
    let bytes = include_bytes!("../char_to_gaiji.map");
    let mut aligned = AlignedVec::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);
    let archived =
        check_archived_root::<RevGaijiMap>(&aligned).expect("char_to_gaiji.map data is corrupted");
    archived.deserialize(&mut Infallible).unwrap()
});

pub fn gaiji_to_char(input: &mut &str) -> Result<char, ContextError> {
    parse_tag(location)
        .verify_map(|t| t.char())
        .parse_next(input)
}

#[test]
fn test() {
    println!(
        "{:?}",
        parse_tag(location)
            .parse_next(&mut "「爿＋戈」、第4水準2-12-83")
            .unwrap()
    );
}
