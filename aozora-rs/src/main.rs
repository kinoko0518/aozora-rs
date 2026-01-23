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
        .decode(include_bytes!("../assets/test_data/oto.txt"))
        .0
        .to_string()
        .replace("\r\n", "\n");

    println!(
        "{:?}",
        scopenize(
            tokenize(&mut LocatingSlice::new(oto.as_str())).unwrap(),
            oto.as_str()
        )
    );
}
