use std::{
    fs::File,
    path::Path,
    time::{Duration, Instant},
};

use aozora_rs::{
    AozoraError, Dependencies,
    internal::{
        AozoraTokenKind, EpubSetting, Note, from_aozora_zip, parse_meta, retokenize,
        retokenized_to_xhtml, scopenize, tokenize,
    },
    utf8tify_all_gaiji,
};
use encoding_rs::SHIFT_JIS;
use serde::Serialize;
use winnow::LocatingSlice;

#[derive(Debug, Serialize, Clone)]
pub struct WorkAnalyse {
    // 作品メタデータ
    pub title: String,
    pub author: String,
    // 作品サイズ
    pub word_count: usize,
    pub byte_count: usize,
    pub token_count: usize,
    pub deco_count: usize,
    // 変換エラー
    pub scopenize_errors: Vec<String>,
    pub retokenize_errors: Vec<String>,
    // 解析エラー
    pub invalid_gaiji: Vec<String>,
    pub invalid_notes: Vec<String>,
    // 各段階の所要時間
    pub read: Duration,
    pub gaiji_convert: Duration,
    pub get_meta: Duration,
    pub tokenize: Duration,
    pub scopenize: Duration,
    pub retokenize: Duration,
    pub xhtml_gen: Duration,
    pub epub_gen: Duration,
}

impl WorkAnalyse {
    pub fn pure_parsetime(&self) -> Duration {
        self.gaiji_convert + self.get_meta + self.tokenize + self.scopenize + self.retokenize
    }

    pub fn total_parsetime(&self) -> Duration {
        self.read
            + self.gaiji_convert
            + self.get_meta
            + self.tokenize
            + self.scopenize
            + self.retokenize
            + self.xhtml_gen
            + self.epub_gen
    }
}

pub fn analyse_per_work(s: &str, base_path: &Path) -> Result<WorkAnalyse, AozoraError> {
    let read_instant = Instant::now();
    let decode = |bytes: &[u8]| -> String {
        let (cow, _, _) = SHIFT_JIS.decode(bytes);
        cow.replace("\r\n", "\n")
    };
    let original_text = decode(&std::fs::read(s).map_err(|e| e.into())?);
    let read_duration = read_instant.elapsed();

    let gaiji_instant = Instant::now();
    let gaiji_converted = utf8tify_all_gaiji(original_text.as_str());
    let gaiji_duration = gaiji_instant.elapsed();
    let (s, invalid_gaijis) = gaiji_converted;
    let mut s_slice = s.as_ref();

    let meta_instant = Instant::now();
    let meta = parse_meta(&mut s_slice).map_err(|e| e.into())?;
    let meta_duration = meta_instant.elapsed();

    let title_owned = meta.title.to_string();
    let author_owned = meta.author.to_string();

    let tokenize_instant = Instant::now();
    let tokenized = tokenize(&mut LocatingSlice::new(s_slice)).map_err(|e| e.into())?;
    let tokenize_duration = tokenize_instant.elapsed();

    let invalid_notes: Vec<String> = tokenized
        .iter()
        .filter_map(|t| match &t.kind {
            AozoraTokenKind::Note(n) => match n {
                Note::Unknown(unknown) => Some(unknown),
                _ => None,
            },
            _ => None,
        })
        .map(|s| s.to_string())
        .collect();
    let deco_count: usize = tokenized
        .iter()
        .filter_map(|s| match s.kind {
            AozoraTokenKind::Note(_) => Some(()),
            AozoraTokenKind::Ruby(_) => Some(()),
            _ => None,
        })
        .count();
    let token_count: usize = tokenized.len();

    let scopenize_instant = Instant::now();
    let ((deco, flat), scopenize_errors) = scopenize(tokenized).into_tuple();
    let scopenized_duration = scopenize_instant.elapsed();

    let retokenize_instant = Instant::now();
    let (retokenized, retokenize_errors) = retokenize(flat, deco).into_tuple();
    let retokenized_duration = retokenize_instant.elapsed();

    let xhtmlnize_instant = Instant::now();
    let xhtmlnized = retokenized_to_xhtml(retokenized);
    let xhtmlnize_duration = xhtmlnize_instant.elapsed();

    let epub_instant = Instant::now();
    let epub_base_path = base_path.join("result/epubs");
    std::fs::create_dir_all(&epub_base_path).map_err(|e| e.into())?;
    from_aozora_zip(
        File::create(epub_base_path.join(format!("{}.epub", title_owned))).map_err(|e| e.into())?,
        &Dependencies::default(),
        &xhtmlnized,
        &EpubSetting {
            styles: vec![
                include_str!("../../../aozora-rs/css/prelude.css"),
                include_str!("../../../aozora-rs/css/vertical.css"),
                include_str!("../../../../ayame/ayame/assets/miyabi.css"),
            ],
            ..Default::default()
        },
        &meta,
        &aozora_rs::PageInjectors::default(),
    )
    .map_err(|e| e.into())?;
    let epub_duration = epub_instant.elapsed();

    Ok(WorkAnalyse {
        title: title_owned,
        author: author_owned,

        word_count: s.chars().count(),
        deco_count,
        token_count,
        byte_count: s.as_bytes().len(),

        scopenize_errors: scopenize_errors
            .iter()
            .map(|s| s.display(s_slice))
            .collect(),
        retokenize_errors: retokenize_errors.iter().map(|s| s.to_string()).collect(),
        invalid_gaiji: invalid_gaijis.iter().map(|s| s.to_string()).collect(),
        invalid_notes,

        read: read_duration,
        gaiji_convert: gaiji_duration,
        get_meta: meta_duration,
        tokenize: tokenize_duration,
        scopenize: scopenized_duration,
        retokenize: retokenized_duration,
        xhtml_gen: xhtmlnize_duration,
        epub_gen: epub_duration,
    })
}
