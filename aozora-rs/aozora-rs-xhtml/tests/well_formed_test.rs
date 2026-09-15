use aozora_rs_core::{Deco, Page, Retokenized};
use aozora_rs_xhtml::retokenized_to_xhtml;

#[test]
fn test_xml_escaping() {
    let special_text = "夏目漱石 & 芥川龍之介 <太宰治> \"走れメロス\" '檸檬'";
    let page = Page {
        content: vec![
            Retokenized::Text(special_text),
            Retokenized::Br,
        ],
        ..Default::default()
    };

    let result = retokenized_to_xhtml(vec![page]);
    assert_eq!(result.xhtmls.len(), 1);

    let xhtml = &result.xhtmls[0];
    println!("Generated XHTML:\n{}", xhtml);

    // 生のアンパサンドや不等号が残っていないことを確認
    assert!(xhtml.contains("&amp;"));
    assert!(xhtml.contains("&lt;太宰治&gt;"));

    // roxmltreeで整形式XMLとしてパースできることを検証
    let doc = roxmltree::Document::parse(xhtml)
        .expect("Generated XHTML must be a well-formed XML document");

    // パース後のテキストノードが元の文字列と一致することを検証（エスケープが正しく復元される）
    let text_nodes: Vec<_> = doc
        .descendants()
        .filter(|n| n.is_text())
        .map(|n| n.text().unwrap())
        .collect();

    assert!(
        text_nodes.iter().any(|t| t.contains("夏目漱石 & 芥川龍之介 <太宰治>")),
        "Parsed text should match original decoded string"
    );
}

#[test]
fn test_heading_inside_indent_well_formed() {
    let page = Page {
        content: vec![
            Retokenized::DecoBegin(Deco::Indent(5)),
            Retokenized::DecoBegin(Deco::BHead),
            Retokenized::Text("第一章　序曲"),
            Retokenized::DecoEnd(Deco::BHead),
            Retokenized::DecoEnd(Deco::Indent(5)),
            Retokenized::Br,
            Retokenized::Text("本文がここから始まります。"),
        ],
        ..Default::default()
    };

    let result = retokenized_to_xhtml(vec![page]);
    assert_eq!(result.xhtmls.len(), 1);

    let xhtml = &result.xhtmls[0];
    println!("Generated Heading XHTML:\n{}", xhtml);

    // roxmltreeで整形式XMLとしてパース
    let doc = roxmltree::Document::parse(xhtml)
        .expect("Heading inside indent must produce well-formed XML");

    // h2要素の中にdiv要素が含まれていない（逆転混入がない）ことを検証
    for node in doc.descendants().filter(|n| n.tag_name().name() == "h2") {
        for child in node.descendants() {
            assert_ne!(
                child.tag_name().name(),
                "div",
                "h2 heading must not contain div block element"
            );
        }
    }
}

#[test]
fn test_ruby_and_annotations_well_formed() {
    let page = Page {
        content: vec![
            Retokenized::DecoBegin(Deco::Ruby("メロス")),
            Retokenized::Text("走れ"),
            Retokenized::DecoEnd(Deco::Ruby("メロス")),
            Retokenized::Br,
            Retokenized::Kunten("一"),
            Retokenized::Text("天地"),
            Retokenized::Okurigana("の"),
            Retokenized::Br,
        ],
        ..Default::default()
    };

    let result = retokenized_to_xhtml(vec![page]);
    assert_eq!(result.xhtmls.len(), 1);

    let xhtml = &result.xhtmls[0];
    println!("Generated Ruby XHTML:\n{}", xhtml);

    let doc = roxmltree::Document::parse(xhtml)
        .expect("Ruby and annotations must produce well-formed XML");

    // ruby要素およびrt要素が存在することを確認
    assert!(doc.descendants().any(|n| n.tag_name().name() == "ruby"));
    assert!(doc.descendants().any(|n| n.tag_name().name() == "rt"));
}

#[test]
fn test_consecutive_line_breaks_well_formed() {
    let page = Page {
        content: vec![
            Retokenized::Text("段落1"),
            Retokenized::Br,
            Retokenized::Br,
            Retokenized::Br,
            Retokenized::Text("段落2"),
        ],
        ..Default::default()
    };

    let result = retokenized_to_xhtml(vec![page]);
    assert_eq!(result.xhtmls.len(), 1);

    let xhtml = &result.xhtmls[0];
    println!("Generated Breaks XHTML:\n{}", xhtml);

    let doc = roxmltree::Document::parse(xhtml)
        .expect("Consecutive line breaks must produce well-formed XML");

    // ルート要素がdivであり、テキストノードが正しく取得できることを確認
    assert_eq!(doc.root_element().tag_name().name(), "div");
    let texts: Vec<_> = doc
        .descendants()
        .filter(|n| n.is_text())
        .map(|n| n.text().unwrap().trim())
        .filter(|t| !t.is_empty())
        .collect();
    assert_eq!(texts, vec!["段落1", "段落2"]);
}

#[test]
fn test_grounded_paragraph_well_formed() {
    let page = Page {
        content: vec![
            Retokenized::DecoBegin(Deco::Grounded),
            Retokenized::Text("（昭和二十三年十二月）"),
            Retokenized::DecoEnd(Deco::Grounded),
        ],
        ..Default::default()
    };

    let result = retokenized_to_xhtml(vec![page]);
    assert_eq!(result.xhtmls.len(), 1);

    let xhtml = &result.xhtmls[0];
    println!("Generated Grounded XHTML:\n{}", xhtml);

    let doc = roxmltree::Document::parse(xhtml)
        .expect("Grounded paragraph must produce well-formed XML");

    // p要素が存在し、class="grounded"属性を持っていることを検証
    let p_node = doc.descendants().find(|n| n.tag_name().name() == "p")
        .expect("p element should be present");
    assert_eq!(p_node.attribute("class"), Some("grounded"));
}

#[test]
fn test_unclosed_tags_auto_recovered() {
    // 終了タグが欠落している（閉じ忘れ）不正な注記列
    let page = Page {
        content: vec![
            Retokenized::DecoBegin(Deco::Indent(3)),
            Retokenized::DecoBegin(Deco::Bold),
            Retokenized::Text("閉じられていない太字と字下げ"),
            // DecoEnd(Deco::Bold) と DecoEnd(Deco::Indent) が抜けている
        ],
        ..Default::default()
    };

    let result = retokenized_to_xhtml(vec![page]);
    assert_eq!(result.xhtmls.len(), 1);

    let xhtml = &result.xhtmls[0];
    println!("Generated Auto-Recovered XHTML:\n{}", xhtml);

    // TreeBuilderにより、未終了のタグが自動的にすべて閉じられ、整形式XMLとして成立することを検証
    let doc = roxmltree::Document::parse(xhtml)
        .expect("TreeBuilder must automatically close all unclosed tags");

    assert!(doc.descendants().any(|n| n.tag_name().name() == "span"));
    assert!(doc.descendants().any(|n| n.tag_name().name() == "div"));
}

#[test]
fn test_ast_type_guarantee() {
    use aozora_rs_xhtml::ast::{BlockNode, HeadingLevel, InlineNode};

    // HeadingはchildrenにInlineNodeしか受け付けない（型システム上の強制）
    let heading = BlockNode::Heading {
        level: HeadingLevel::H2,
        attributes: vec!["class=\"b_head\"".into()],
        children: vec![
            InlineNode::Text("見出しテキスト".into()),
            InlineNode::Span {
                attributes: vec!["class=\"sub\"".into()],
                children: vec![InlineNode::Text("（副題）".into())],
            },
        ],
    };

    let mut output = String::new();
    heading.render(&mut output, 0);

    let doc = roxmltree::Document::parse(&output)
        .expect("Directly rendered AST must be valid XML");
    assert_eq!(doc.root_element().tag_name().name(), "h2");
}


