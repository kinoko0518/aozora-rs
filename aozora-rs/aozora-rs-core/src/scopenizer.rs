//! ルビや注記などの装飾指定トークンとその周囲のトークンの情報から
//! 注記の適用範囲を確定、どのように記述されていたかの情報を単純化します。

mod conversion;
mod definition;
mod error;
mod parser;
#[cfg(test)]
mod test;

pub use crate::scopenizer::definition::*;
pub use crate::scopenizer::error::ScopenizeError;
pub use crate::scopenizer::parser::scopenize;
