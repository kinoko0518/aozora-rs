use crate::xhtmlnize::definitions::{XHTMLKind, XHTMLTag};

pub fn validate_xhtml<'s>(buff: Vec<XHTMLTag<'s>>) -> Vec<XHTMLTag<'s>> {
    // XHTML正規化ルール
    // - 見かけの改行の数は保持されなければならない
    //  - 補足：<br>はコンテナ一つの開始・終了タグと等価
    // - テキストやルビなどのインライン要素はいずれかのコンテナの中に存在していなければならない
    // - インライン要素がいずれのコンテナにも含まれていない場合<p></p>で囲む
    // - <div>、<h1>（以後、大局コンテナ）などの要素の中に<p>は存在できない

    let mut peekable = buff.into_iter().peekable();
    let mut buff = Vec::new();

    // コンテナの種類を追跡するための列挙型を定義
    #[derive(PartialEq)]
    enum ContainerKind {
        Block,
        P,
    }
    let mut stack: Vec<ContainerKind> = Vec::new();

    while let Some(current) = peekable.next() {
        if current.kind.is_block_begin() {
            stack.push(ContainerKind::Block);
            buff.push(current);
            // 次がBrなら消費する
            if let Some(next) = peekable.peek() {
                if matches!(next.kind, XHTMLKind::Br) {
                    peekable.next();
                }
            }
            continue;
        }

        if current.kind.is_block_end() {
            stack.pop();
            buff.push(current);
            continue;
        }

        if let XHTMLKind::PBegin = current.kind {
            stack.push(ContainerKind::P);
            buff.push(current);
            continue;
        }

        if let XHTMLKind::PEnd = current.kind {
            stack.pop();
            buff.push(current);
            continue;
        }

        // [br]のルール
        if let XHTMLKind::Br = current.kind {
            let next_is_inline = peekable.peek().map_or(false, |s| s.kind.is_inline());
            let next_is_br = peekable
                .peek()
                .map_or(false, |s| matches!(s.kind, XHTMLKind::Br));

            if stack.is_empty() {
                if !next_is_inline || next_is_br {
                    // 空行用のコンテナに変化させる (<p><br /></p>)
                    buff.push(XHTMLTag::from_kind(XHTMLKind::PBegin));
                    buff.push(XHTMLTag::from_kind(XHTMLKind::Br));
                    buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                } else {
                    // 自身を<p>開始タグに変化させる
                    buff.push(XHTMLTag::from_kind(XHTMLKind::PBegin));
                    stack.push(ContainerKind::P);
                }
            } else {
                // すでにコンテナ内にいる場合はそのまま追加
                buff.push(XHTMLTag::from_kind(XHTMLKind::Br));
            }
            continue;
        }

        if current.kind.is_inline() {
            // いずれのコンテナの中にもいなければ直前に<p>を追加
            if stack.is_empty() {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PBegin));
                stack.push(ContainerKind::P);
            }

            buff.push(current);

            // 次の要素がインライン要素でも Br でもない場合、直近の親が<p>なら閉じる
            let next_is_inline_or_br = peekable.peek().map_or(false, |s| {
                s.kind.is_inline() || matches!(s.kind, XHTMLKind::Br)
            });

            if !next_is_inline_or_br {
                if let Some(ContainerKind::P) = stack.last() {
                    buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                    stack.pop();
                }
            }
            continue;
        }

        // いずれも当てはまらなければそのまま追加
        buff.push(current);
    }

    // 処理の最後に<p>が閉じられていなければ閉じる
    if let Some(ContainerKind::P) = stack.last() {
        buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
        stack.pop();
    }

    buff
}
