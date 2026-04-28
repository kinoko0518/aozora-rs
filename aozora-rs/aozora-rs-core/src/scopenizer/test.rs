use winnow::LocatingSlice;

use crate::{BotenKind, Deco, Scope, ScopenizeError, scopenize, tokenize};

fn easy_scopenize<'s>(input: &'s str) -> (Scope<'s>, Vec<ScopenizeError>) {
    let tokenized = tokenize(&mut LocatingSlice::new(input)).unwrap();
    let ((scopenized, _), err) = scopenize(tokenized).into_tuple();
    let (_, scope) = scopenized.0.into_iter().next().unwrap();
    (scope.into_iter().next().unwrap(), err)
}

#[test]
fn backref_test() {
    let (scope, err) = easy_scopenize("えて［＃「えて」に傍点］して");

    assert!(err.is_empty());
    assert_eq!(
        scope,
        Scope {
            deco: Deco::Boten(BotenKind::Sesame),
            span: 0..6
        }
    );
}

#[test]
fn rubyref_test() {
    let (scope, err) = easy_scopenize("ひらがな漢字《かんじ》");

    assert!(err.is_empty());
    assert_eq!(
        scope,
        Scope {
            deco: Deco::Ruby("かんじ"),
            span: 12..18
        }
    );
}

#[test]
fn wholeline_test() {
    let (scope, err) = easy_scopenize("［＃地付き］令和８年４月２８日");
    assert!(err.is_empty());
    let begin: usize = "［＃地付き］".chars().map(|c| c.len_utf8()).sum();
    let end = begin
        + "令和８年４月２８日"
            .chars()
            .map(|c| c.len_utf8())
            .sum::<usize>();
    assert_eq!(
        scope,
        Scope {
            deco: Deco::Grounded,
            span: begin..end
        }
    )
}

#[test]
fn sandwiched_test() {
    let (scope, err) = easy_scopenize("［＃傍点］ヒロソヒイ［＃傍点終わり］");

    assert!(err.is_empty());
    assert_eq!(
        scope,
        Scope {
            deco: Deco::Boten(BotenKind::Sesame),
            span: 15..30
        }
    )
}

#[test]
fn multiline_test() {
    let (scope, err) = easy_scopenize(
        "［＃ここから２字下げ］\n秋の田の\nかりほの庵の\n苫を荒み\nわがころも手は\n露に濡れつつ\n［＃ここで字下げ終わり］\n",
    );
    let tag: usize = "［＃ここから２字下げ］\n"
        .chars()
        .map(|c| c.len_utf8())
        .sum();
    let main: usize = "秋の田の\nかりほの庵の\n苫を荒み\nわがころも手は\n露に濡れつつ"
        .chars()
        .map(|c| c.len_utf8())
        .sum();

    assert!(err.is_empty());
    assert_eq!(
        scope,
        Scope {
            deco: Deco::Indent(2),
            span: tag..(tag + main)
        }
    );
}
