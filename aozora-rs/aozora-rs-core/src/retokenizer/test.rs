use winnow::LocatingSlice;

use crate::{Deco, Retokenized, retokenize, scopenize, tokenize};

#[test]
fn kyusoku() {
    let input = "［＃２字下げ］休息［＃「休息」は中見出し］";

    let tokenized = tokenize(&mut LocatingSlice::new(input)).unwrap();
    let ((scope, exps), serr) = scopenize(tokenized).into_tuple();
    let (pages, rerr) = retokenize(exps, scope);

    assert_eq!(serr, vec![]);
    assert_eq!(rerr, vec![]);
    assert_eq!(
        pages.first().unwrap().content,
        vec![
            Retokenized::DecoBegin(Deco::BHead),
            Retokenized::DecoBegin(Deco::Indent(2)),
            Retokenized::Text("休息"),
            Retokenized::DecoEnd(Deco::Indent(2)),
            Retokenized::DecoEnd(Deco::BHead),
        ]
    )
}
