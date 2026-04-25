//! エラー関連の定義を行うモジュールです。

mod azerror;
mod decolated;

pub use azerror::{AZResult, AZResultC};
pub(crate) use decolated::display_error_with_decolation;
