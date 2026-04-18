mod errors;
mod style;

use std::io::{Seek, Write};

pub mod internal {
    pub use aozora_rs_core::*;
    pub use aozora_rs_epub::{EpubSetting, from_aozora_zip};
    pub use aozora_rs_gaiji::*;
    pub use aozora_rs_xhtml::retokenized_to_xhtml;
    pub use aozora_rs_zip::{Dependencies, Encoding};
}

use internal::*;

pub use aozora_rs_epub::{PageInjectors, TitlePageHyle, TocPageHyle};
pub use aozora_rs_gaiji::{gaiji_to_char, utf8tify_all_gaiji};
pub use aozora_rs_xhtml::{Chapter, XHTMLResult};
pub use aozora_rs_zip::AozoraZip;
pub use style::{Style, WritingDirection};

pub use errors::*;
use winnow::LocatingSlice;

pub struct AozoraDocument<'s> {
    pub meta: AozoraMeta<'s>,
    pub text: &'s str,
    dependencies: Option<&'s Dependencies>,
}

impl<'s> TryFrom<&'s AozoraZip> for AozoraDocument<'s> {
    type Error = AozoraError;
    fn try_from(value: &'s AozoraZip) -> Result<Self, Self::Error> {
        let (meta, text) = str_to_meta_and_str(&value.txt.as_str())?;
        Ok(Self {
            meta,
            text,
            dependencies: Some(&value.images),
        })
    }
}

fn str_to_xhtml(text: &str) -> Result<(XHTMLResult, Vec<AozoraWarning>), AozoraError> {
    let mut loc = LocatingSlice::new(text);
    let tokenized = tokenize(&mut loc).map_err(|e| e.into())?;
    let ((scopenized, flattoken), scopenized_err) = scopenize(tokenized).into_tuple();
    let (retokenized, retokenized_err) = retokenize(flattoken, scopenized).into_tuple();
    let xhtml_result = retokenized_to_xhtml(retokenized);
    let warn = scopenized_err
        .into_iter()
        .map(|err| err.into())
        .chain(retokenized_err.into_iter().map(|e| e.into()));
    Ok((xhtml_result, warn.collect()))
}

fn str_to_meta_and_str<'s>(text: &'s str) -> Result<(AozoraMeta<'s>, &'s str), AozoraError> {
    let mut cursor = &*text;
    let meta = parse_meta(&mut cursor).map_err(|e| e.into())?;
    Ok((meta, cursor))
}

impl<'s> AozoraDocument<'s> {
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

    pub fn from_zip(zip: &'s AozoraZip) -> Result<Self, AozoraError> {
        Self::try_from(zip)
    }

    pub fn xhtml(&self) -> Result<(XHTMLResult, Vec<AozoraWarning>), AozoraError> {
        str_to_xhtml(self.text)
    }

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
            &style.into_epub_setting(),
            &self.meta,
            injectors,
        )
        .map_err(|e| e.into())?
        .into_tuple();
        warn.extend(zip_warn.into_iter().map(|w| w.into()));
        Ok(warn)
    }
}
