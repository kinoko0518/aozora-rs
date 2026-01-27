use std::{borrow::Cow, fs::File, io::Write};

use crate::convert::into_xhtml;

mod convert;

fn main() {
    let (encoded, _, _) = encoding_rs::SHIFT_JIS.decode(include_bytes!("../example/oto.txt"));
    let mut exported = File::create("./output.html").unwrap();
    writeln!(
        exported,
        "{}",
        aozora_rs::parse(&encoded.replace("\r\n", "\n"))
            .unwrap()
            .1
            .into_iter()
            .map(|t| into_xhtml(t).0)
            .collect::<Vec<Cow<'_, str>>>()
            .join("")
    )
    .unwrap();
}
