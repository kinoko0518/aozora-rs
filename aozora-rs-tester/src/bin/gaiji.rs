use std::path::{Path, PathBuf};

use aozora_rs_gaiji::gaiji_to_char;
use aozora_rs_tester::AnalysedData;
use aozora_rs_tester::update_map;
use futures::stream::{self, StreamExt};
use winnow::{
    Parser,
    combinator::{delimited, repeat, repeat_till},
    error::ContextError,
    token::{any, take_until},
};

async fn analyse_file(path: &Path) -> Option<AnalysedData> {
    let read_bin = std::fs::read(path).ok()?;
    let mut read_raw: &str = &encoding_rs::SHIFT_JIS.decode(&read_bin).0;
    let read = &mut read_raw;
    let gaiji_note = delimited("※［＃", take_until(0.., "］"), "］");
    let gaiji_notes = repeat(
        0..,
        repeat_till(0.., any::<_, ContextError>.void(), gaiji_note).map(|(_, s): ((), &str)| s),
    );

    let found = gaiji_notes
        .fold(
            || AnalysedData::new(),
            |mut acc, mut e: &str| {
                if let Some(_) = gaiji_to_char(&mut e) {
                    acc.success();
                } else {
                    acc.fail(e.to_string());
                }
                acc
            },
        )
        .parse_next(read)
        .ok()?;
    Some(found)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = update_map(&std::env::current_dir()?)?;
    let result = stream::iter(map.paths)
        .map(|path| tokio::spawn(async move { analyse_file(&PathBuf::from(path)).await }))
        .buffer_unordered(100)
        .filter_map(async move |s| s.ok().flatten())
        .fold(
            AnalysedData::new(),
            async move |acc: AnalysedData, d: AnalysedData| acc.join(d),
        )
        .await;
    println!("\n----------- 解析完了！ -----------\n");
    let failed_len = result.failed.iter().fold(0_usize, |acc, v| {
        println!("パースに失敗しました：{}", v);
        acc + 1
    });
    println!(
        "\n成功：{}件 失敗{}件 成功率{}%",
        result.successed,
        failed_len,
        (result.successed as f32) / (result.successed + failed_len) as f32 * 100.0
    );
    Ok(())
}
