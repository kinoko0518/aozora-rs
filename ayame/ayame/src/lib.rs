use std::io::Write;

pub use aozora_rs::{
    AozoraDocument, AozoraError, AozoraWarning, AozoraZip, Chapter, PageInjectors, Style,
    TitlePageHyle, TocPageHyle, WritingDirection, XHTMLResult, utf8tify_all_gaiji,
};
pub use aozora_rs::{Dependencies, Encoding};

pub const MIYABI_CSS: &str = include_str!("../assets/miyabi.css");

/// miyabi CSSをStyleに適用
pub fn apply_miyabi(style: &mut Style) {
    style.add_css(MIYABI_CSS);
}

/// ブラウザ表示可能な完全XHTMLファイルを生成
pub fn to_browser_xhtml(
    doc: &AozoraDocument,
    style: &Style,
) -> Result<(String, Vec<AozoraWarning>), AozoraError> {
    let (xhtml_result, warnings) = doc.xhtml()?;
    let css = style.css();
    let css_combined = css.join("\n");
    let body = xhtml_result.xhtmls.join("\n<hr>\n");

    let result = include_str!("../assets/base.xhtml")
        .replace("［＃タイトル］", doc.meta.title)
        .replace("［＃スタイル］", &css_combined)
        .replace("［＃本文］", &body);

    Ok((result, warnings))
}

/// 扉ページの書き込みロジックを生成
pub fn title_page_writer() -> Box<dyn Fn(&mut dyn Write, &TitlePageHyle) -> std::io::Result<()>> {
    Box::new(|writer, hyle| {
        writeln!(writer, "<h1 class=\"title\">{}</h1>", hyle.title)?;
        writeln!(writer, "<p class=\"author\">{}</p>", hyle.author)?;
        Ok(())
    })
}

/// 目次ページの書き込みロジックを生成
pub fn toc_page_writer() -> Box<dyn Fn(&mut dyn Write, &TocPageHyle) -> std::io::Result<()>> {
    Box::new(|writer, hyle| {
        writer.write_all("<h1 class=\"toc-title\">目　次</h1>\n".as_bytes())?;
        writer.write_all(b"<ol class=\"toc-list\">\n")?;
        for chapter in hyle.chapters {
            writeln!(
                writer,
                "\t<li><a href=\"sec{:>04}.xhtml#{}\">{}</a></li>",
                chapter.xhtml_id,
                chapter.get_id(),
                chapter.name
            )?;
        }
        writer.write_all(b"</ol>\n")?;
        Ok(())
    })
}
