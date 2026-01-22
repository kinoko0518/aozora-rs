mod command;
mod definition;

use winnow::combinator::{alt, repeat};

type Span = std::ops::Range<usize>;

pub fn tokenize<'s>(mut text: &'s str) -> Vec<AozoraToken<'s>> {
    repeat(alt(()))
}
