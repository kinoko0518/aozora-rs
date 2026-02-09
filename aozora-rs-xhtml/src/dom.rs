use aozora_rs_core::prelude::{Break, Deco, Retokenized};
use itertools::Itertools;

use crate::definitions::{CDepth, Chapter};

pub struct MappedToken<'s> {
    pub content: Retokenized<'s>,
    pub chapter: Option<Chapter>,
}

pub struct Mapped<'s> {
    pub xhtmls: Vec<Vec<MappedToken<'s>>>,
    pub dependency: Vec<&'s str>,
}

/// Retokenizedトークン列を改ページで分割、トークンにXHTML変換時特有の情報を付加する関数です。
pub fn into_mapped<'s>(retokenized: Vec<Retokenized<'s>>) -> Mapped<'s> {
    let mut xhtmls = Vec::new();
    let mut buff = Vec::new();

    let mut iter = retokenized.into_iter().multipeek();
    let mut cdepth = CDepth::new();

    let mut dependency = Vec::new();

    while let Some(s) = iter.next() {
        // 改ページ、または大見出しが来たらXHTMLを分割
        if let Retokenized::Break(Break::PageBreak) | Retokenized::DecoBegin(Deco::AHead) = s {
            xhtmls.push(std::mem::take(&mut buff));
        }
        // (大|中|小)見出しならChapterを構築
        macro_rules! parse_chapter {
            ($deco_variant:path, $inc_method:ident) => {{
                // CDepthを更新
                cdepth.$inc_method();
                let mut buff = String::new();

                // 章の名前を探索
                while let Some(s) = iter.peek() {
                    match s {
                        Retokenized::DecoEnd($deco_variant) => {
                            break;
                        }
                        Retokenized::Text(t) => {
                            buff.extend(t.chars());
                        }
                        _ => (),
                    }
                }
                iter.reset_peek();
                Some(Chapter {
                    xhtml_id: xhtmls.len(),
                    name: buff,
                    depth: cdepth.clone(),
                })
            }};
        }
        let chapter = match s {
            Retokenized::DecoBegin(Deco::AHead) => {
                parse_chapter!(Deco::AHead, increament_a)
            }
            Retokenized::DecoBegin(Deco::BHead) => {
                parse_chapter!(Deco::BHead, increament_b)
            }
            Retokenized::DecoBegin(Deco::CHead) => {
                parse_chapter!(Deco::CHead, increament_c)
            }
            _ => None,
        };

        if let Retokenized::Figure(f) = &s {
            dependency.push(f.path);
        }
        buff.push(MappedToken {
            // Figureなら依存関係を追加
            content: s,
            chapter: chapter,
        });
    }

    // 最後にbuffに中身が残っていればフラッシュ
    if !buff.is_empty() {
        xhtmls.push(buff);
    }

    Mapped { xhtmls, dependency }
}
