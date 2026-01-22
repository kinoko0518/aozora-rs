//! aozorahackの青空文庫書式のドキュメントに記載されているところの、開始/終了型のパースを行うモジュールです。
//!　ドキュメントは[こちら](https://github.com/aozorahack/specs/blob/master/aozora-text.md#%E5%89%8D%E6%96%B9%E5%8F%82%E7%85%A7%E5%9E%8B%E3%81%A8%E9%96%8B%E5%A7%8B%E7%B5%82%E4%BA%86%E5%9E%8B)
//! から確認できます。

use crate::tokenizer::Span;
use crate::tokenizer::command::{SandwichedBegin, definitions::*};

struct Bold {
    pub span: Span,
}
struct Italic {
    pub span: Span,
}
#[derive(PartialEq, Eq)]
struct Bosen {
    pub span: Span,
    pub kind: BosenKind,
}
#[derive(PartialEq, Eq)]
struct Boten {
    pub span: Span,
    pub kind: BotenKind,
}

impl_sandwiched!(SandwichedEnds, Bold, BoldEnd);
impl_sandwiched!(SandwichedEnds, Italic, ItalicEnd);

impl SandwichedBegin<SandwichedEnds> for Boten {
    fn effect_range(&self, rhs: &SandwichedEnds) -> Option<Span> {
        if let SandwichedEnds::BotenEnd(k) = rhs
            && self == k
        {
            Some((self.span.end + 1)..(k.span.start - 1))
        } else {
            None
        }
    }
}
impl SandwichedBegin<SandwichedEnds> for Bosen {
    fn effect_range(&self, rhs: &SandwichedEnds) -> Option<Span> {
        if let SandwichedEnds::BosenEnd(k) = rhs
            && self == k
        {
            Some((self.span.end + 1)..(k.span.start - 1))
        } else {
            None
        }
    }
}

#[enum_dispatch::enum_dispatch]
enum SandwichedBegins {
    BoldBegin(Bold),
    ItalicBegin(Italic),
    BotenBegin(Boten),
    BosenBegin(Bosen),
}

enum SandwichedEnds {
    BoldEnd(Span),
    ItalicEnd(Span),
    BotenEnd(Boten),
    BosenEnd(Bosen),
}

pub enum Sandwiched {
    Begin(SandwichedBegins),
    End(SandwichedEnds),
}
