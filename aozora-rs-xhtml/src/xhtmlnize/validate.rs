use crate::xhtmlnize::definitions::{XHTMLKind, XHTMLTag};

pub fn validate_xhtml<'s>(buff: Vec<XHTMLTag<'s>>) -> Vec<XHTMLTag<'s>> {
    let mut peekable = buff.into_iter().peekable();
    let mut buff = Vec::new();

    #[derive(PartialEq)]
    enum ContainerKind {
        Block,
        P,
    }
    let mut stack: Vec<ContainerKind> = Vec::new();

    while let Some(current) = peekable.next() {
        if current.kind.is_block_begin() {
            // 新しいブロックを開始する際、スタックのトップが<p>であれば先に閉じる
            if let Some(ContainerKind::P) = stack.last() {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                stack.pop();
            }

            stack.push(ContainerKind::Block);
            buff.push(current);
            // 次がBrなら消費する
            if peekable
                .peek()
                .is_some_and(|next| matches!(next.kind, XHTMLKind::Br))
            {
                peekable.next();
            }
            continue;
        }

        if current.kind.is_block_end() {
            // ブロックを終了する際、スタックのトップが<p>であれば先に閉じる
            if let Some(ContainerKind::P) = stack.last() {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                stack.pop();
            }

            stack.pop(); // Blockをポップ
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

        if let XHTMLKind::Br = current.kind {
            let next_is_inline = peekable.peek().is_some_and(|s| s.kind.is_inline());
            let next_is_br = peekable
                .peek()
                .is_some_and(|s| matches!(s.kind, XHTMLKind::Br));

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
                    buff.push(current);
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
            let next_is_inline_or_br = peekable
                .peek()
                .is_some_and(|s| s.kind.is_inline() || matches!(s.kind, XHTMLKind::Br));

            if !next_is_inline_or_br && let Some(ContainerKind::P) = stack.last() {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                stack.pop();
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
