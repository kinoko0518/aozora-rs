use std::borrow::Cow;

use crate::convert::into_xhtml;
use aozora_rs_core::prelude::*;
use wasm_bindgen::prelude::*;
use winnow::LocatingSlice;

mod convert;

#[wasm_bindgen]
pub struct NovelResult {
    #[wasm_bindgen(getter_with_clone)]
    pub title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub author: String,
    #[wasm_bindgen(getter_with_clone)]
    pub body: String,
}

#[wasm_bindgen]
/// 一行目はタイトル、二行目は著者名、【テキスト中に現れる記号について】を無視といった
/// 特別な表記を一切考慮せず、入力すべてに対して常に一般的なルールに基づきパースを行います。
///
/// 自身のサイトの中に青空文庫書式で書いたテキストをHTMLとして埋め込みたいときなどにご活用ください。
pub fn convert_with_no_meta(input: &str) -> String {
    let mut input_slice = LocatingSlice::new(input);
    let tokens = tokenize_nometa(&mut input_slice).unwrap();
    let ((scopenized, flattoken), _) = scopenize(tokens, input).into_tuple();
    let retokenized = retokenize(flattoken, scopenized);

    retokenized
        .into_iter()
        .map(|t| into_xhtml(t).0)
        .collect::<Vec<Cow<'_, str>>>()
        .join("")
}

#[wasm_bindgen]
/// 一行目はタイトル、二行目は著者名、【テキスト中に現れる記号について】を無視といった
/// 特別な表記を考慮し、メタデータとして解析します。
///
/// 既存の青空文庫書式で書かれた作品をパースして独自の表示を行いたいときなどに有用です。
pub fn convert_with_meta(input: &str) -> NovelResult {
    let (meta, parsed) = aozora_rs_core::parse(input).unwrap();
    let main_text = parsed
        .into_iter()
        .map(|t| into_xhtml(t).0)
        .collect::<Vec<Cow<'_, str>>>()
        .join("");

    NovelResult {
        title: meta.title.to_string(),
        author: meta.author.to_string(),
        body: main_text,
    }
}

#[cfg(feature = "debug")]
#[wasm_bindgen]
/// メタデータを考慮し、cssなどを同梱してある程度綺麗に読める状態に整ったHTMLを取得できます。
///
/// もともとパーサーの動作検証用ですが、何か用途があるかもしれないので公開しています。
pub fn convert_debug(input: &str) -> String {
    let (meta, parsed) = aozora_rs::parse(input).unwrap();
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

    format!(
        "{}",
        include_str!("../assets/debug.html")
            .replace("<!-- タイトル -->", meta.title)
            .replace("<!-- 著者 -->", meta.author)
            .replace("<!-- スタイル -->", &joined_style)
            .replace("<!-- 本文 -->", &main_text)
    )
}
