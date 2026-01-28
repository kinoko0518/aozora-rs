use std::{borrow::Cow, fs::File, io::Write};

use crate::convert::into_xhtml;

mod convert;

fn main() -> Result<(), miette::Error> {
    let (encoded, _, _) = encoding_rs::SHIFT_JIS.decode(include_bytes!("../example/shayo.txt"));
    let cleansed = encoded.replace("\r\n", "\n");
    let (meta, parsed) = aozora_rs::parse(&cleansed)?;
    let mut exported = File::create(format!("./[{}] {}.html", meta.author, meta.title)).unwrap();

    writeln!(
        exported,
        "{}",
        parsed
            .into_iter()
            .map(|t| into_xhtml(t).0)
            .collect::<Vec<Cow<'_, str>>>()
            .join("")
    )
    .unwrap();
    Ok(())
}
