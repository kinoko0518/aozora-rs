mod gaiji_chuki;
mod menkuten;

use std::{env, path::Path};

use menkuten::satisfy_latest_menkuten;
use winnow::{Parser, ascii::*, combinator::*, error::ContextError, token::take_while};

use crate::gaiji_chuki::{get_latest_gaiji_chuki, satisfy_pdfium};

/// 16進数で表現された数字にマッチし、charに変換して値を返す
pub fn hex_digit_as_char(input: &mut &str) -> Result<char, ContextError> {
    hex_digit1
        .map(|digit| u32::from_str_radix(digit, 16).unwrap())
        .map(|digit| char::from_u32(digit).unwrap())
        .parse_next(input)
}

/// U+12EFやU+12EF+34CDのような文字列にマッチし、それぞれの4桁の数字をUnicodeと解釈してStringに集約する
pub fn parse_single_utf8(input: &mut &str) -> Result<String, ContextError> {
    let ex_code = ('+', hex_digit_as_char).map(|(_, code): (_, char)| code);
    ("U", repeat(0.., ex_code))
        .map(|(_, multi): (_, String)| multi)
        .parse_next(input)
}

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
