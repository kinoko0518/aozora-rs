use winnow::LocatingSlice;

use crate::{Deco, Retokenized, retokenize, scopenize, tokenize};

#[test]
fn kyusoku() {
    let input = "［＃２字下げ］休息［＃「休息」は中見出し］";

    let tokenized = tokenize(&mut LocatingSlice::new(input)).unwrap();
    let ((scope, flatt), serr) = scopenize(tokenized).into_tuple();
    let (retokenized, rerr) = retokenize(flatt, scope).into_tuple();

    assert!(serr.is_empty() && rerr.is_empty());
    assert_eq!(
        retokenized,
        vec![
            Retokenized::DecoBegin(Deco::BHead),
            Retokenized::DecoBegin(Deco::Indent(2)),
            Retokenized::Text("休息"),
            Retokenized::DecoEnd(Deco::Indent(2)),
            Retokenized::DecoEnd(Deco::BHead),
        ]
    )
}
