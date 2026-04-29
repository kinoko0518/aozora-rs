use crate::{retokenizer::processor::RetokenizeEvent, *};

/// 明示的に区切られたページを表現します。
pub struct Page<'s> {
    /// ページ内の要素です。
    pub content: Vec<Retokenized<'s>>,
    /// ページの中身が中央寄せされているかを表します。
    pub is_centre: bool,
    /// ページの要素が必ず左から開始する、右から開始するなどを指定します。
    pub page_begin: PageBegin,
}

impl Default for Page<'_> {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            is_centre: false,
            page_begin: PageBegin::Whatever,
        }
    }
}

impl<'s> Page<'s> {
    pub(crate) fn push(&mut self, content: Retokenized<'s>) {
        self.content.push(content);
    }

    pub(crate) fn retokenize(
        &mut self,
        events: &mut impl Iterator<Item = (usize, RetokenizeEvent<'s>)>,
    ) -> Vec<RetokenizeError> {
        // 蓄積用
        let mut err = Vec::new();

        // 閉じられていないトークン
        let mut unclosed_token = (Option::None, 0);
        // 閉じられていないタグ
        let mut unclosed_decos: Vec<Deco<'s>> = Vec::new();

        fn warn_at_end<'s>(
            err: &mut Vec<RetokenizeError>,
            ut: (Option<Element>, usize),
            ud: Vec<Deco<'s>>,
        ) {
            if let (Some(_), _) = ut {
                err.push(RetokenizeError::InvalidEndOfToken);
            }
            err.extend([RetokenizeError::InvalidEndOfScope].repeat(ud.len()));
        }

        for (i, event) in events.by_ref() {
            match event {
                // 平坦トークン開始
                RetokenizeEvent::FlatTBegin(f) => {
                    unclosed_token = (Some(f), i);
                }
                // 平坦トークン終了
                RetokenizeEvent::FlatTEnd => match unclosed_token.0 {
                    Some(t) => {
                        // 閉じる
                        self.push(t.into());
                        unclosed_token = (None, i)
                    }
                    None => err.push(
                        // 開始されていないものを閉じたらエラーを蓄積
                        RetokenizeError::InvalidEndOfToken,
                    ),
                },
                RetokenizeEvent::DecoBegin(d) => {
                    if let (Some(t), unclosed_until) = unclosed_token {
                        // 閉じられていないトークンがあれば一次終了して装飾を開始
                        // トークンを現在位置で分割
                        let (confirmed, unclosed) = t.split_at(i - unclosed_until);
                        // 分割以前を確定、開始タグを挿入、分割以後で未確定トークンを再開
                        self.push(confirmed.into());
                        self.push(Retokenized::DecoBegin(d.clone()));
                        unclosed_decos.push(d);
                        unclosed_token = (unclosed, i);
                    } else {
                        // 閉じられていないトークンが無かったときは単純に開始タグを挿入
                        self.push(Retokenized::DecoBegin(d.clone()));
                        unclosed_decos.push(d);
                    }
                }
                RetokenizeEvent::DecoEnd => {
                    // 装飾を終了
                    let popped_deco = unclosed_decos.pop();

                    if let (Some(t), unclosed_until) = unclosed_token {
                        // 閉じられていないトークンがあったら
                        if let Some(d) = popped_deco {
                            // 閉じられていないトークンをポップ
                            // 分割して分割以前を確定、終了タグを挿入して分割以後を再開
                            let (confirmed, unclosed) = t.split_at(i - unclosed_until);
                            self.push(confirmed.into());
                            self.push(Retokenized::DecoEnd(d));
                            unclosed_token = (unclosed, i);
                        } else {
                            // 開始されなかったスコープを終了しようとしたらエラー
                            err.push(RetokenizeError::InvalidEndOfScope);
                            unclosed_token = (Some(t), unclosed_until);
                        }
                    } else {
                        // 閉じられていないトークンがない
                        if let Some(d) = popped_deco {
                            // 閉じられていない装飾があれば単に装飾タグを挿入
                            self.push(Retokenized::DecoEnd(d));
                        } else {
                            // 閉じられていない装飾もなければエラー
                            err.push(RetokenizeError::InvalidEndOfScope);
                        }
                    }
                }
                // 改ページがあれば終了
                RetokenizeEvent::PageBreak => {
                    warn_at_end(&mut err, unclosed_token, unclosed_decos);
                    return err;
                }
                // ページ定義があれば自身をmutate
                RetokenizeEvent::PageDef(d) => match d {
                    PageDef::FromLeft => self.page_begin = PageBegin::Left,
                    PageDef::FromRight => self.page_begin = PageBegin::Right,
                    PageDef::VHCentre => self.is_centre = true,
                },
            }
        }
        warn_at_end(&mut err, unclosed_token, unclosed_decos);
        err
    }
}
