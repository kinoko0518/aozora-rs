// 再エクスポート
pub use aozora_rs_core::*;
pub use aozora_rs_epub::{
    AozoraZip, AozoraZipError, EpubSetting, from_aozora_zip, from_sjis_aozora_zip,
    from_utf8_aozora_zip,
};
pub use aozora_rs_xhtml::{NovelResult, XHTMLResult, retokenized_to_xhtml};
use winnow::{LocatingSlice, error::ContextError};

/// 青空文庫書式で記述されたテキストを直接中間表現である[Retokenized]に変換します。
///
/// メタデータを考慮する場合、先に[parse_meta]を実行してください。
/// [Tokenized]、[Scopenized]、[ScopeKind]が必要な場合は各関数を順番に適用してください。
pub fn str_to_retokenized<'s>(
    str: &'s str,
) -> Result<AZResult<Vec<Retokenized<'s>>>, ContextError> {
    let tokenized = tokenize(&mut LocatingSlice::new(str))?;
    let ((deco, flat), errors) = scopenize(tokenized, str).into_tuple();
    let retokenized = retokenize(flat, deco);
    Ok(AZResultC::from(errors).finally(retokenized))
}
