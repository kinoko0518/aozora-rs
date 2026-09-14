use crate::xhtmlnize::definitions::{XHTMLKind, XHTMLTag};

pub fn validate_xhtml<'s>(buff: Vec<XHTMLTag<'s>>) -> Vec<XHTMLTag<'s>> {
    let mut peekable = buff.into_iter().peekable();
    let mut buff = Vec::new();

    #[derive(PartialEq, Debug)]
    enum ContainerKind {
        Block,
        Heading,
        ExplicitP,
        ImplicitP,
    }
    let mut stack: Vec<ContainerKind> = Vec::new();

    while let Some(current) = peekable.next() {
        if current.kind.is_block_begin() {
            // 新しいブロックを開始する際、スタックのトップが暗黙の<p>であれば先に閉じる
            if let Some(ContainerKind::ImplicitP) = stack.last() {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                stack.pop();
            }

            match current.kind {
                XHTMLKind::PBegin => stack.push(ContainerKind::ExplicitP),
                XHTMLKind::H1Begin | XHTMLKind::H2Begin | XHTMLKind::H3Begin => {
                    stack.push(ContainerKind::Heading)
                }
                _ => {
                    if let Some(ContainerKind::Heading) = stack.last() {
                        buff.push(XHTMLTag {
                            kind: XHTMLKind::SpanBegin,
                            attributes: current.attributes,
                        });
                        continue;
                    }
                    stack.push(ContainerKind::Block);
                }
            }

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
            // ブロックを終了する際、スタックのトップが暗黙の<p>で自身がPEndでなければ先に閉じる
            if let Some(ContainerKind::ImplicitP) = stack.last()
                && !matches!(current.kind, XHTMLKind::PEnd)
            {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                stack.pop();
            }

            if matches!(current.kind, XHTMLKind::DivEnd)
                && let Some(ContainerKind::Heading) = stack.last()
            {
                buff.push(XHTMLTag::from_kind(XHTMLKind::SpanEnd));
                continue;
            }

            if !stack.is_empty() {
                stack.pop();
            }
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
                    stack.push(ContainerKind::ImplicitP);
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
                stack.push(ContainerKind::ImplicitP);
            }

            buff.push(current);

            // 次の要素がインライン要素でも Br でもない場合、直近の親が暗黙の<p>なら閉じる
            let next_is_inline_or_br = peekable
                .peek()
                .is_some_and(|s| s.kind.is_inline() || matches!(s.kind, XHTMLKind::Br));

            if !next_is_inline_or_br && let Some(ContainerKind::ImplicitP) = stack.last() {
                buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
                stack.pop();
            }
            continue;
        }

        // いずれも当てはまらなければそのまま追加
        buff.push(current);
    }

    // 処理の最後に閉じられていない<p>があれば閉じる
    if let Some(ContainerKind::ImplicitP | ContainerKind::ExplicitP) = stack.last() {
        buff.push(XHTMLTag::from_kind(XHTMLKind::PEnd));
        stack.pop();
    }

    buff
}
