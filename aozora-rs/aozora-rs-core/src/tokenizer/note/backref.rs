#![doc = include_str!("../../../docs/note/backref.md")]

use winnow::{
    Parser,
    combinator::{alt, delimited},
    token::take_until,
};

use crate::{nihongo::japanese_num, tokenizer::definition::*, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackRefSpec<'s>(pub &'s str);

/// 前方参照型注記の直和です。
///
/// ［＃「……」は……］というように、直接影響範囲を記述する形式に対応します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackRefKind<'s> {
    /// 太字
    Bold,
    /// 斜体
    Italic,
    /// 傍点
    Boten(BotenKind),
    /// 傍線
    Bosen(BosenKind),
    /// 大見出し
    AHead,
    /// 中見出し
    BHead,
    /// 小見出し
    CHead,
    /// ママ
    Mama,
    /// 注記
    Note(&'s str),
    /// 縦中横
    HinV,
    /// N段階小さな文字
    Small(usize),
    /// N段階大きな文字
    Big(usize),
    /// …はXXでは
    Variation((&'s str, &'s str)),
    /// 下付き小文字
    Sub,
    /// 上付き小文字
    Sup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackRef<'s> {
    pub kind: BackRefKind<'s>,
    pub range: BackRefSpec<'s>,
}

impl<'s> Into<AozoraTokenKind<'s>> for BackRef<'s> {
    fn into(self) -> AozoraTokenKind<'s> {
        AozoraTokenKind::Note(Note::BackRef(self))
    }
}

pub fn backref<'s>(input: &mut Input<'s>) -> Result<BackRef<'s>, WinnowError> {
    let target = BackRefSpec(delimited("「", take_until(1.., "」"), "」").parse_next(input)?);
    let ha = (
        'は',
        alt((
            "太字".value(BackRefKind::Bold),
            "斜体".value(BackRefKind::Italic),
            "大見出し".value(BackRefKind::AHead),
            "中見出し".value(BackRefKind::BHead),
            "小見出し".value(BackRefKind::CHead),
            "ママ".value(BackRefKind::Mama),
            alt(("縦中横", "横一列")).value(BackRefKind::HinV),
            (japanese_num, "段階小さな文字").map(|(size, _)| BackRefKind::Small(size)),
            (japanese_num, "段階大きな文字").map(|(size, _)| BackRefKind::Big(size)),
            (
                take_until(1.., "では"),
                "では「",
                take_until(0.., "」"),
                "」",
            )
                .map(|(on, _, variation, _)| BackRefKind::Variation((on, variation))),
            alt(("下付き小文字", "行右小書き")).value(BackRefKind::Sub),
            alt(("上付き小文字", "行左小書き")).value(BackRefKind::Sup),
        )),
    )
        .map(|(_, v)| v);
    let ni = (
        'に',
        alt((
            alt((bosen.map(BackRefKind::Bosen), boten.map(BackRefKind::Boten))),
            ('「', take_until(1.., '」'), '」', "の注記").map(|(_, s, _, _)| BackRefKind::Note(s)),
        )),
    )
        .map(|(_, v)| v);
    alt((ha, ni))
        .map(|b| BackRef {
            kind: b,
            range: target,
        })
        .parse_next(input)
}
