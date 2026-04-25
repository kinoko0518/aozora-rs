#![doc = include_str!("../../../docs/note/single.md")]

use winnow::{
    Parser,
    combinator::{alt, delimited, opt},
    token::{one_of, take_until},
};

use crate::{nihongo::japanese_num, *};

/// 注記分類のうちの1つ、単一表現型注記の直和です。
///
/// それ単体で完結して表示できる注記が分類されます。
#[derive(Debug, Clone)]
pub enum Single<'s> {
    /// 「改ページ」に対応
    PageBreak,
    /// 「改丁」に対応
    RectoBreak,
    /// 「改見開き」に対応
    SpreadBreak,
    /// 「改段」に対応
    ColumnBreak,
    /// 図に対応
    Figure(Figure<'s>),
    /// 訓読文字に対応
    Kundoku(&'s str),
    /// 漢文の送り仮名に対応
    Okurigana(&'s str),
}

fn figure_size(input: &mut Input) -> Result<(usize, usize), WinnowError> {
    (
        "横",
        japanese_num,
        one_of(('×', 'x', 'X', 'X')),
        "縦",
        japanese_num,
    )
        .map(|(_, w, _, _, h)| (w, h))
        .parse_next(input)
}

pub(crate) fn single<'s>(input: &mut Input<'s>) -> Result<Single<'s>, WinnowError> {
    let path_and_size = (
        take_until(1.., "、"),
        opt(("、", figure_size).map(|(_, size)| size)),
    );
    let figure = (
        take_until(0.., "（"),
        delimited("（", path_and_size, "）"),
        "入る",
    )
        .map(|(caption, (path, size), _)| Figure {
            path,
            caption,
            size,
        });
    alt((
        alt(("改ページ", "改頁")).value(Single::PageBreak),
        "改丁".value(Single::RectoBreak),
        "改段".value(Single::ColumnBreak),
        "改見開き".value(Single::SpreadBreak),
        // 訓読文字
        "一レ".value(Single::Kundoku("一レ")),
        "上レ".value(Single::Kundoku("一レ")),
        "甲レ".value(Single::Kundoku("一レ")),
        'レ'.value(Single::Kundoku("レ")),
        '一'.value(Single::Kundoku("一")),
        '二'.value(Single::Kundoku("二")),
        '三'.value(Single::Kundoku("三")),
        '上'.value(Single::Kundoku("上")),
        '中'.value(Single::Kundoku("中")),
        '下'.value(Single::Kundoku("下")),
        '甲'.value(Single::Kundoku("上")),
        '乙'.value(Single::Kundoku("中")),
        '丙'.value(Single::Kundoku("丙")),
        ('（', take_until(1.., '）'), '）').map(|(_, txt, _)| Single::Okurigana(txt)),
        figure.map(Single::Figure),
    ))
    .parse_next(input)
}
