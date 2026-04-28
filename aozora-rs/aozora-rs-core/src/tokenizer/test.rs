use winnow::LocatingSlice;

use crate::{
    AozoraTokenKind, BackRefKind, Figure, WholeLine, tokenize,
    tokenizer::annotation::backref::{BackRef, BackRefSpec},
};

fn easy_tokenkind<'s>(input: &'s str) -> Vec<AozoraTokenKind<'s>> {
    tokenize(&mut LocatingSlice::new(input))
        .unwrap()
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

#[test]
fn br_test() {
    let tokenized =
        easy_tokenkind("ちはやぶる\n神代も聞かず\n竜田川\nからくれなゐに\n水くくるとは");
    assert_eq!(
        tokenized,
        vec![
            AozoraTokenKind::Text("ちはやぶる"),
            AozoraTokenKind::Br,
            AozoraTokenKind::Text("神代も聞かず"),
            AozoraTokenKind::Br,
            AozoraTokenKind::Text("竜田川"),
            AozoraTokenKind::Br,
            AozoraTokenKind::Text("からくれなゐに"),
            AozoraTokenKind::Br,
            AozoraTokenKind::Text("水くくるとは")
        ]
    )
}

#[test]
fn ruby_test() {
    let tokenized =
        easy_tokenkind("｜私は何を知るか《ク・セ・ジュ》？、とモンテーニュは問《と》うた。");
    assert_eq!(
        tokenized,
        vec![
            AozoraTokenKind::RubyDelimiter,
            AozoraTokenKind::Text("私は何を知るか"),
            AozoraTokenKind::Ruby("ク・セ・ジュ"),
            AozoraTokenKind::Text("？、とモンテーニュは問"),
            AozoraTokenKind::Ruby("と"),
            AozoraTokenKind::Text("うた。")
        ]
    );
}

#[test]
fn num_in_annotations() {
    let tokenized = easy_tokenkind("［＃6字下げ］［＃６字下げ］");

    assert_eq!(
        tokenized,
        vec![WholeLine::Indent(6).into(), WholeLine::Indent(6).into()]
    )
}

#[test]
fn backref_annotations() {
    let tokenized = easy_tokenkind("青空分庫［＃「青空分庫」はママ］");

    assert_eq!(
        tokenized,
        vec![
            AozoraTokenKind::Text("青空分庫"),
            BackRef {
                kind: BackRefKind::Mama,
                range: BackRefSpec("青空分庫")
            }
            .into()
        ]
    );
}

#[test]
fn image_insertion() {
    let tokenized = easy_tokenkind("［＃コンドル博士の図（fig47728_06.png、横320×縦322）入る］");

    assert_eq!(
        tokenized,
        vec![
            Figure {
                path: "fig47728_06.png",
                caption: "コンドル博士の図",
                size: Some((320, 322))
            }
            .into()
        ]
    )
}
