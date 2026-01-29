use std::{borrow::Cow, fs::File, io::Write};

use crate::convert::into_xhtml;

mod convert;

fn main() -> Result<(), miette::Error> {
    let (encoded, _, _) =
        encoding_rs::SHIFT_JIS.decode(include_bytes!("../example/haruto_shura.txt"));
    let cleansed = encoded.replace("\r\n", "\n");
    let (meta, parsed) = aozora_rs::parse(&cleansed).unwrap();
    let mut exported = File::create(format!("./[{}] {}.html", meta.author, meta.title)).unwrap();
    let joined_style = format!(
        "<style>{}\n{}\n{}</style>",
        include_str!("../style/aozora.css"),
        include_str!("../style/boten.css"),
        include_str!("../style/bosen.css"),
    );
    let main_text = parsed
        .into_iter()
        .map(|t| into_xhtml(t).0)
        .collect::<Vec<Cow<'_, str>>>()
        .join("");

    writeln!(
        exported,
        "{}",
        include_str!("../assets/debug.html")
            .replace("<!-- タイトル -->", meta.title)
            .replace("<!-- 著者 -->", meta.author)
            .replace("<!-- スタイル -->", &joined_style)
            .replace("<!-- 本文 -->", &main_text)
    )
    .unwrap();
    Ok(())
}
