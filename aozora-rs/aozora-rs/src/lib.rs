//! aozora-rsのファザードクレートです。青空文庫のパースから別の形式への変換を行えます。

#![warn(missing_docs)]

mod errors;
mod style;

use std::io::{Seek, Write};

/// 内部的な型やメソッドにアクセスするためのモジュールです。
///
/// internal内の型やメソッドの定義は更新により頻繫に変更される可能性があるため、利用はunrecommendedです。
pub mod internal {
    pub use aozora_rs_core::*;
    pub use aozora_rs_epub::{EpubSetting, from_aozora_zip};
    pub use aozora_rs_gaiji::*;
    pub use aozora_rs_xhtml::retokenized_to_xhtml;
}

use internal::*;

pub use aozora_rs_epub::{PageInjectors, TitlePageHyle, TocPageHyle};
pub use aozora_rs_gaiji::{gaiji_to_char, utf8tify_all_gaiji};
pub use aozora_rs_xhtml::{Chapter, XHTMLResult};
pub use aozora_rs_zip::AozoraZip;
pub use aozora_rs_zip::{Dependencies, Encoding};
pub use style::{Style, WritingDirection};

pub use errors::*;
use winnow::LocatingSlice;

/// EPUB、XHTML、その他の出力形式に変換されるために必要なメタデータ、本文、依存関係のデータを持つ構造体です。
///
/// # Example
/// ```
/// use aozora_rs::AozoraDocument;
///
/// let doc = AozoraDocument::from_str(
///     "\
///     『春と修羅』\n\
///     宮沢賢治\n\
///     \n\
///     -------------------------------------------------------\n\
///     【テキスト中に現れる記号について】\n\
///     ...\n\
///     -------------------------------------------------------\n\
///     \n\
///     ［＃２字下げ］序［＃「序」は大見出し］\n
///     \n\
///     \n\
///     わたくしといふ現象は\n\
///     仮定された有機交流電燈の\n\
///     ひとつの青い照明です\n\
///     あらゆる透明な幽霊の複合体\n\
///     ",
///     None
/// ).unwrap();
/// let (xhtml, _) = doc.xhtml().unwrap();
/// println!("{:?}", xhtml.xhtmls.get(0).unwrap());
/// ```
pub struct AozoraDocument<'s> {
    /// メタデータを格納します。
    pub meta: AozoraMeta<'s>,
    /// 本文（メタデータを除く部分）を格納します。
    pub text: &'s str,
    dependencies: Option<&'s Dependencies>,
}

impl<'s> TryFrom<&'s AozoraZip> for AozoraDocument<'s> {
    type Error = AozoraError;
    fn try_from(value: &'s AozoraZip) -> Result<Self, Self::Error> {
        let (meta, text) = str_to_meta_and_str(value.txt.as_str())?;
        Ok(Self {
            meta,
            text,
            dependencies: Some(&value.images),
        })
    }
}

fn str_to_xhtml(text: &str) -> Result<(XHTMLResult, Vec<AozoraWarning>), AozoraError> {
    let mut loc = LocatingSlice::new(text);
    let tokenized = tokenize(&mut loc).map_err(AozoraError::from)?;
    let ((scopenized, flattoken), scopenized_err) = scopenize(tokenized).into_tuple();
    let (retokenized, retokenized_err) = retokenize(flattoken, scopenized);
    let xhtml_result = retokenized_to_xhtml(retokenized);
    let warn = scopenized_err
        .into_iter()
        .map(|err| err.into())
        .chain(retokenized_err.into_iter().map(|e| e.into()));
    Ok((xhtml_result, warn.collect()))
}

fn str_to_meta_and_str<'s>(text: &'s str) -> Result<(AozoraMeta<'s>, &'s str), AozoraError> {
    let mut cursor = text;
    let meta = parse_meta(&mut cursor).map_err(AozoraError::from)?;
    Ok((meta, cursor))
}

impl<'s> AozoraDocument<'s> {
    /// [`&str`]と[`Option<&Dependencies>`]から[`AozoraDocument`]を構築します。
    /// 特に画像などの依存関係が無い場合Noneを渡してください。
    pub fn from_str(
        text: &'s str,
        dependencies: Option<&'s Dependencies>,
    ) -> Result<Self, AozoraError> {
        let (meta, text) = str_to_meta_and_str(text)?;
        Ok(Self {
            meta,
            text,
            dependencies,
        })
    }

    /// メタデータと本文を直接注入して[`AozoraDocument`]を構築します。
    ///
    /// textのヘッダにメタデータが記述されていても考慮されません。すでにパースしたメタデータを注入したり、
    /// モックのメタデータを注入する用途を想定しています。
    pub fn from_str_and_meta(
        meta: AozoraMeta<'s>,
        text: &'s str,
        dependencies: Option<&'s Dependencies>,
    ) -> Self {
        Self {
            meta,
            text,
            dependencies,
        }
    }

    /// [`&AozoraZip`]から[`AozoraDocument`]を構築します。
    ///
    /// # Example
    /// ```
    /// use aozora_rs::AozoraZip;
    /// use std::io::Cursor;
    /// use aozora_rs::Encoding;
    /// use aozora_rs::AozoraDocument;
    ///
    /// let azz = AozoraZip::read_from_zip(
    ///     Cursor::new(include_bytes!("../example/anausa_peterno_hanasi.zip")),
    ///     &Encoding::ShiftJIS
    /// ).unwrap();
    /// AozoraDocument::from_zip(&azz).unwrap();
    /// ```
    pub fn from_zip(zip: &'s AozoraZip) -> Result<Self, AozoraError> {
        Self::try_from(zip)
    }

    /// 自身のデータからXHTMLを構築して返します。
    pub fn xhtml(&self) -> Result<(XHTMLResult, Vec<AozoraWarning>), AozoraError> {
        str_to_xhtml(self.text)
    }

    /// 自身のデータからEPUBを構築し、writerに書き込みます。
    ///
    /// writerには書き込み先、styleには縦書き・横書き、カスタムCSS、言語コードなどのデータを内包する[`Style`]を受け取ります。
    /// injectorsには目次や扉ページの組み立てロジックをまとめた型、[`PageInjectors`]を要求します。
    ///
    ///　目次、章ページが不要な場合は[`PageInjectors::default`]を利用できます。
    ///
    /// # Example
    /// ```
    /// use aozora_rs::{
    ///     AozoraDocument, Style,
    ///     WritingDirection, PageInjectors,
    /// };
    ///
    /// let doc = AozoraDocument::from_str(
    ///     "\
    ///     『春と修羅』\n\
    ///     宮沢賢治\n\
    ///     \n\
    ///     -------------------------------------------------------\n\
    ///     【テキスト中に現れる記号について】\n\
    ///     ...\n\
    ///     -------------------------------------------------------\n\
    ///     \n\
    ///     ［＃２字下げ］序［＃「序」は大見出し］\n
    ///     \n\
    ///     \n\
    ///     わたくしといふ現象は\n\
    ///     仮定された有機交流電燈の\n\
    ///     ひとつの青い照明です\n\
    ///     あらゆる透明な幽霊の複合体\n\
    ///     ",
    ///     None
    /// ).unwrap();
    /// let mut cursor = std::io::Cursor::new(Vec::new());
    /// let injector = PageInjectors::default();
    /// let _ = doc.epub(
    ///     &mut cursor,
    ///     &Style::default()
    ///         .direction(WritingDirection::Vertical)
    ///         .language("ja"),
    ///     &injector
    /// ).unwrap();
    /// ```
    pub fn epub<T>(
        &self,
        writer: &mut T,
        style: &Style,
        injectors: &PageInjectors,
    ) -> Result<Vec<AozoraWarning>, AozoraError>
    where
        T: Write + Seek,
    {
        let dependencies = match self.dependencies {
            Some(s) => s,
            None => &Dependencies::default(),
        };
        let (xhtml, mut warn) = self.xhtml()?;
        let ((), zip_warn) = aozora_rs_epub::from_aozora_zip(
            writer,
            dependencies,
            &xhtml,
            &style.epub_setting(),
            &self.meta,
            injectors,
        )
        .map_err(AozoraError::from)?
        .into_tuple();
        warn.extend(zip_warn.into_iter().map(|w| w.into()));
        Ok(warn)
    }
}
