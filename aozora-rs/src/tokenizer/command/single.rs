use winnow::{
    Parser,
    combinator::{alt, delimited, opt},
    error::ContextError,
    token::{one_of, take_until},
};

use crate::{nihongo::japanese_num, prelude::*};

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
}

fn figure_size(input: &mut Input) -> Result<(usize, usize), ContextError> {
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

pub fn single<'s>(input: &mut Input<'s>) -> Result<Single<'s>, ContextError> {
    let path_and_size = (
        take_until(1.., "、"),
        opt(("、", figure_size).map(|(_, size)| size)),
    );
    let figure = (take_until(0.., "（"), delimited("（", path_and_size, "）")).map(
        |(caption, (path, size))| Figure {
            path: path,
            caption: caption,
            size: size,
        },
    );
    alt((
        "改ページ".value(Single::PageBreak),
        "改丁".value(Single::RectoBreak),
        "改段".value(Single::ColumnBreak),
        "改見開き".value(Single::SpreadBreak),
        figure.map(Single::Figure),
    ))
    .parse_next(input)
}
