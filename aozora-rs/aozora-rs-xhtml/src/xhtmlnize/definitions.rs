use std::borrow::Cow;
use std::fmt::Write;

pub enum XHTMLKind<'s> {
    Text(&'s str),
    SpanBegin,
    SpanEnd,
    H1Begin,
    H1End,
    H2Begin,
    H2End,
    H3Begin,
    H3End,
    Br,
    DivBegin,
    DivEnd,
    SubBegin,
    SubEnd,
    SupBegin,
    SupEnd,
    RubyBegin,
    RubyEnd,
    RtBegin,
    RtEnd,
    Img,
    PBegin,
    PEnd,
}

impl XHTMLKind<'_> {
    /// <p>の中に存在できるか
    pub fn is_inline(&self) -> bool {
        match self {
            Self::Text(_) => true,
            Self::SpanBegin => true,
            Self::SpanEnd => true,
            Self::H1Begin => false,
            Self::H1End => false,
            Self::H2Begin => false,
            Self::H2End => false,
            Self::H3Begin => false,
            Self::H3End => false,
            Self::Br => true,
            Self::DivBegin => false,
            Self::DivEnd => false,
            Self::SupBegin => true,
            Self::SupEnd => true,
            Self::SubBegin => true,
            Self::SubEnd => true,
            Self::RubyBegin => true,
            Self::RubyEnd => true,
            Self::RtBegin => true,
            Self::RtEnd => true,
            Self::Img => false,
            Self::PBegin => false,
            Self::PEnd => false,
        }
    }

    /// 単体で存在することができるか
    pub fn is_single(&self) -> bool {
        match self {
            Self::Br => true,
            Self::Img => true,
            _ => false,
        }
    }

    pub fn is_block_begin(&self) -> bool {
        matches!(
            self,
            XHTMLKind::H1Begin
                | XHTMLKind::H2Begin
                | XHTMLKind::H3Begin
                | XHTMLKind::DivBegin
                | XHTMLKind::PBegin
        )
    }

    pub fn is_block_end(&self) -> bool {
        matches!(
            self,
            XHTMLKind::H1End
                | XHTMLKind::H2End
                | XHTMLKind::H3End
                | XHTMLKind::DivEnd
                | XHTMLKind::PEnd
        )
    }
}

pub struct XHTMLTag<'s> {
    pub kind: XHTMLKind<'s>,
    pub attributes: Vec<Cow<'s, str>>,
}

impl<'s> XHTMLTag<'s> {
    pub fn from_kind(kind: XHTMLKind<'s>) -> Self {
        Self {
            kind,
            attributes: Vec::new(),
        }
    }

    pub fn into_htmltag(self) -> Cow<'s, str> {
        let mut buff = String::from("<");
        buff.push_str(match self.kind {
            XHTMLKind::Text(t) => {
                return Cow::Borrowed(t);
            }
            XHTMLKind::Br => "br",
            XHTMLKind::DivBegin => "div",
            XHTMLKind::DivEnd => "/div",
            XHTMLKind::H1Begin => "h1",
            XHTMLKind::H1End => "/h1",
            XHTMLKind::H2Begin => "h2",
            XHTMLKind::H2End => "/h2",
            XHTMLKind::H3Begin => "h3",
            XHTMLKind::H3End => "/h3",
            XHTMLKind::Img => "img",
            XHTMLKind::PBegin => "p",
            XHTMLKind::PEnd => "/p",
            XHTMLKind::RtBegin => "rt",
            XHTMLKind::RtEnd => "/rt",
            XHTMLKind::RubyBegin => "ruby",
            XHTMLKind::RubyEnd => "/ruby",
            XHTMLKind::SpanBegin => "span",
            XHTMLKind::SpanEnd => "/span",
            XHTMLKind::SupBegin => "sup",
            XHTMLKind::SupEnd => "/sup",
            XHTMLKind::SubBegin => "sub",
            XHTMLKind::SubEnd => "/sub",
        });
        for atr in &self.attributes {
            write!(buff, " ").unwrap();
            write!(buff, "{}", atr).unwrap();
        }
        if self.kind.is_single() {
            write!(buff, " /").unwrap();
        }
        write!(buff, ">").unwrap();
        Cow::Owned(buff)
    }
}
