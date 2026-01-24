mod deco;
mod nihongo;
mod scopenizer;
mod tokenizer;

use encoding_rs::SHIFT_JIS;
use winnow::LocatingSlice;

use crate::scopenizer::prelude::*;
use crate::tokenizer::prelude::*;

pub mod prelude {
    use winnow::LocatingSlice;

    pub type Input<'s> = LocatingSlice<&'s str>;
    pub type Span = std::ops::Range<usize>;

    pub use crate::deco::*;
}
use crate::prelude::*;

fn main() {
    let oto: String = SHIFT_JIS
        .decode(include_bytes!("../assets/test_data/tsumito_batsu.txt"))
        .0
        .to_string()
        .replace("\r\n", "\n");

    let tokens = tokenize(&mut LocatingSlice::new(oto.as_str())).unwrap();

    println!(
        "{}",
        scopenize(tokens, oto.as_str())
            .unwrap()
            .1
            .iter()
            .filter(|s| !matches!(s.deco, Deco::Ruby(_)))
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("")
    );
}
