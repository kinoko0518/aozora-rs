// 再エクスポート
pub use aozora_rs_core::*;
pub use aozora_rs_epub::{EpubSetting, from_aozora_zip};
pub use aozora_rs_gaiji::*;
pub use aozora_rs_xhtml::{NovelResult, XHTMLResult, into_xhtml, retokenized_to_novel_result};
use winnow::{LocatingSlice, error::ContextError};

/// 青空文庫書式で記述されたテキストを直接中間表現である[Retokenized]に変換します。
///
/// メタデータを考慮する場合は先に[parse_meta]を、外字を考慮する場合、先に[whole_gaiji_to_char]を実行してください。
/// [Tokenized]、[Scopenized]、[ScopeKind]が必要な場合は各関数を順番に適用してください。
pub fn str_to_retokenized<'s>(
    str: &'s str,
) -> Result<AZResult<Vec<Retokenized<'s>>>, ContextError> {
    let tokenized = tokenize(&mut LocatingSlice::new(str))?;
    let ((deco, flat), mut errors) = scopenize(tokenized, str).into_tuple();
    let (retokenized, retokenize_errors) = retokenize(flat, deco).into_tuple();
    errors.extend(retokenize_errors.into_iter());
    Ok(AZResultC::from(errors).finally(retokenized))
}
