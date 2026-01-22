use crate::tokenizer::Span;

enum Single {
    PageBreak(Span),
    RectoBreak(Span),
    SpreadBreak(Span),
    ColumnBreak(Span),
}
