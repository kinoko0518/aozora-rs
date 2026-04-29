use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, InsertTextFormat, TextEdit,
};

use crate::document::DocumentState;

/// `［＃`の直後に提示する補完候補を生成する。
/// `prefix`は`［＃`の後にすでに入力されている文字列。
/// `replace_range`は補完テキストで置き換える範囲。
pub fn compute_completions(
    _doc: &DocumentState,
    prefix: &str,
    replace_range: tower_lsp::lsp_types::Range,
) -> Vec<CompletionItem> {
    let all_items = all_completion_items(replace_range);
    if prefix.is_empty() {
        all_items
    } else {
        all_items
            .into_iter()
            .filter(|item| item.label.starts_with(prefix))
            .collect()
    }
}

fn item(
    label: &str,
    insert: &str,
    detail: &str,
    kind: CompletionItemKind,
    is_snippet: bool,
    replace_range: tower_lsp::lsp_types::Range,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        insert_text_format: if is_snippet {
            Some(InsertTextFormat::SNIPPET)
        } else {
            Some(InsertTextFormat::PLAIN_TEXT)
        },
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: replace_range,
            new_text: insert.to_string(),
        })),
        ..Default::default()
    }
}

fn all_completion_items(r: tower_lsp::lsp_types::Range) -> Vec<CompletionItem> {
    let kw = CompletionItemKind::KEYWORD;
    let snip = CompletionItemKind::SNIPPET;
    let ev = CompletionItemKind::EVENT;

    vec![
        // 改ページ系
        item("改ページ", "改ページ］", "ページを区切る", ev, false, r),
        item("改丁", "改丁］", "左ページから再開", ev, false, r),
        item("改見開き", "改見開き］", "右ページから再開", ev, false, r),
        item("改段", "改段］", "段を区切る", ev, false, r),
        // 行頭型
        item("字下げ", "${1:３}字下げ］", "行をN字下げ", snip, true, r),
        item("地付き", "地付き］", "行を右端揃え", kw, false, r),
        item("地から字上げ", "地から${1:３}字上げ］", "行を右端からN字戻す", snip, true, r),
        // 装飾開始（Sandwiched Begin）
        item("太字", "太字］", "太字開始", kw, false, r),
        item("斜体", "斜体］", "斜体開始", kw, false, r),
        item("傍点", "傍点］", "白ゴマ傍点開始", kw, false, r),
        item("白丸傍点", "白丸傍点］", "白丸傍点開始", kw, false, r),
        item("丸傍点", "丸傍点］", "丸傍点開始", kw, false, r),
        item("白三角傍点", "白三角傍点］", "白三角傍点開始", kw, false, r),
        item("黒三角傍点", "黒三角傍点］", "黒三角傍点開始", kw, false, r),
        item("二重丸傍点", "二重丸傍点］", "二重丸傍点開始", kw, false, r),
        item("蛇の目傍点", "蛇の目傍点］", "蛇の目傍点開始", kw, false, r),
        item("ばつ傍点", "ばつ傍点］", "ばつ傍点開始", kw, false, r),
        item("傍線", "傍線］", "傍線開始", kw, false, r),
        item("二重傍線", "二重傍線］", "二重傍線開始", kw, false, r),
        item("鎖線", "鎖線］", "鎖線開始", kw, false, r),
        item("破線", "破線］", "破線開始", kw, false, r),
        item("波線", "波線］", "波線開始", kw, false, r),
        item("大見出し", "大見出し］", "大見出し開始", kw, false, r),
        item("中見出し", "中見出し］", "中見出し開始", kw, false, r),
        item("小見出し", "小見出し］", "小見出し開始", kw, false, r),
        item("割り注", "割り注］", "割り注開始", kw, false, r),
        item("横組み", "横組み］", "横組み開始", kw, false, r),
        // 装飾終了（Sandwiched End）
        item("太字終わり", "太字終わり］", "太字終了", kw, false, r),
        item("斜体終わり", "斜体終わり］", "斜体終了", kw, false, r),
        item("傍点終わり", "傍点終わり］", "傍点終了", kw, false, r),
        item("傍線終わり", "傍線終わり］", "傍線終了", kw, false, r),
        item("二重傍線終わり", "二重傍線終わり］", "二重傍線終了", kw, false, r),
        item("鎖線終わり", "鎖線終わり］", "鎖線終了", kw, false, r),
        item("破線終わり", "破線終わり］", "破線終了", kw, false, r),
        item("波線終わり", "波線終わり］", "波線終了", kw, false, r),
        item("大見出し終わり", "大見出し終わり］", "大見出し終了", kw, false, r),
        item("中見出し終わり", "中見出し終わり］", "中見出し終了", kw, false, r),
        item("小見出し終わり", "小見出し終わり］", "小見出し終了", kw, false, r),
        item("割り注終わり", "割り注終わり］", "割り注終了", kw, false, r),
        item("横組み終わり", "横組み終わり］", "横組み終了", kw, false, r),
        item("小さな文字終わり", "小さな文字終わり］", "小さな文字終了", kw, false, r),
        item("大きな文字終わり", "大きな文字終わり］", "大きな文字終了", kw, false, r),
        // 複数行ブロック開始（Multiline Begin）
        item("ここから字下げ", "ここから${1:３}字下げ］", "ブロック字下げ開始", snip, true, r),
        item(
            "ここから字下げ（折り返し）",
            "ここから${1:３}字下げ、折り返して${2:２}字下げ］",
            "ぶら下げ字下げ開始",
            snip,
            true,
            r,
        ),
        item("ここから地付き", "ここから地付き］", "ブロック地付き開始", kw, false, r),
        item("ここから字上げ", "ここから地から${1:３}字上げ］", "ブロック地上げ開始", snip, true, r),
        item(
            "ここから小さな文字",
            "ここから${1:２}段階小さな文字］",
            "ブロック小文字開始",
            snip,
            true,
            r,
        ),
        item(
            "ここから大きな文字",
            "ここから${1:２}段階大きな文字］",
            "ブロック大文字開始",
            snip,
            true,
            r,
        ),
        item("ここから字詰め", "ここから${1:２}字詰め］", "ブロック字詰め開始", snip, true, r),
        // 複数行ブロック終了（Multiline End）
        item("ここで字下げ終わり", "ここで字下げ終わり］", "ブロック字下げ終了", kw, false, r),
        item("ここで地付き終わり", "ここで地付き終わり］", "ブロック地付き終了", kw, false, r),
        item("ここで字上げ終わり", "ここで字上げ終わり］", "ブロック地上げ終了", kw, false, r),
        item("ここで小さな文字終わり", "ここで小さな文字終わり］", "ブロック小文字終了", kw, false, r),
        item("ここで大きな文字終わり", "ここで大きな文字終わり］", "ブロック大文字終了", kw, false, r),
        item("ここで字詰め終わり", "ここで字詰め終わり］", "ブロック字詰め終了", kw, false, r),
        // ページ定義
        item("ページの左右中央", "ページの左右中央］", "ページ中央寄せ", kw, false, r),
    ]
}
