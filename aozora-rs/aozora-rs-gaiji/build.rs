mod gaiji_chuki;
mod menkuten;

use std::{env, path::Path};

use menkuten::satisfy_latest_menkuten;
use winnow::{Parser, error::ContextError, token::take_while};

use crate::gaiji_chuki::{get_latest_gaiji_chuki, satisfy_pdfium};

/// 行末まで文字列を読み切りる
fn ignore_rest_of_line<'s>(input: &mut &'s str) -> Result<&'s str, ContextError> {
    take_while(0.., |c| c != '\n' && c != '\r').parse_next(input)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir_env = env::var_os("OUT_DIR").expect("OUT_DIR is not set");
    let out_dir = Path::new(&out_dir_env);

    let menkuten = satisfy_latest_menkuten(out_dir).await?;
    let pdfium = satisfy_pdfium(out_dir).await?;
    get_latest_gaiji_chuki(pdfium, &menkuten, out_dir).await?;

    Ok(())
}
