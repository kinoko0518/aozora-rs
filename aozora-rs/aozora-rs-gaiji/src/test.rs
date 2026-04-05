use gaiji_chuki_parser::GaijiChuki;

use crate::{GAIJI_TO_CHAR, MENKUTEN_TO_UNICODE, parse_tag};

#[test]
fn simple_parse_tag_test() {
    assert_eq!(
        parse_tag(&mut "「木＋雲」、第 4水準 2-15-55").unwrap(),
        GaijiChuki {
            tag: "「木＋雲」",
            sjis: Some((2, 15, 55)),
            unicode: None
        }
    );
    assert_eq!(
        parse_tag(&mut "「木＋犀」、U+6A28、ページ数-行数").unwrap(),
        GaijiChuki {
            tag: "「木＋犀」",
            sjis: None,
            unicode: Some("樨".into())
        }
    );
}

#[test]
fn with_kome_parse_tag() {
    assert_eq!(
        parse_tag(&mut "「※」は「木＋雲」").unwrap(),
        GaijiChuki {
            tag: "「木＋雲」",
            sjis: None,
            unicode: None
        }
    );
}

#[test]
fn with_specific_appeared_parse_tag() {
    assert_eq!(
        parse_tag(&mut "「木＋雲」、8-10").unwrap(),
        GaijiChuki {
            tag: "「木＋雲」",
            sjis: None,
            unicode: None
        }
    );
    assert_eq!(
        parse_tag(&mut "「木＋雲」、11-上-45").unwrap(),
        GaijiChuki {
            tag: "「木＋雲」",
            sjis: None,
            unicode: None
        }
    );
}

#[test]
fn convertion_test() {
    println!(
        "{:?}",
        parse_tag(&mut "「木＋雲」、第 4水準 2-15-55")
            .unwrap()
            .to_cow(&GAIJI_TO_CHAR, &MENKUTEN_TO_UNICODE)
            .unwrap()
    )
}
