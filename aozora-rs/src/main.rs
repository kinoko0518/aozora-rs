mod nihongo;
mod tokenizer;

use encoding_rs::SHIFT_JIS;
use winnow::LocatingSlice;

use crate::tokenizer::tokenize;

type Input<'s> = LocatingSlice<&'s str>;

fn main() {
    let oto: String = SHIFT_JIS
        .decode(include_bytes!("../assets/test_data/oto.txt"))
        .0
        .to_string()
        .replace("\r\n", "\n");

    println!("{:?}", tokenize(&mut LocatingSlice::new(oto.as_str())));
}
