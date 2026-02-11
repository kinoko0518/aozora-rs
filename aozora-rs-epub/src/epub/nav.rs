use std::io::Write;

use crate::epub::EpubWriter;

impl EpubWriter<'_> {
    fn write_nav_head(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"<head>\n")?;
        writer.write_all(b"\t<meta charset=\"UTF-8\" />\n")?;
        writeln!(writer, "\t<title>{}</title>", &self.vzip.nresult.title)?;
        writer.write_all(b"\t<style type=\"text/css\">\n")?;

        self.apply_css(writer)?;

        writer.write_all(b"\t</style>\n")?;
        writer.write_all(b"</head>\n")?;

        Ok(())
    }

    fn write_nav_landmarks(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"\t<nav epub:type=\"landmarks\" id=\"landmarks\" hidden=\"\">\n")?;
        writer.write_all(b"\t\t<h2>Guide</h2>\n")?;
        writer.write_all(b"\t\t<ol>\n")?;
        writer.write_all(
            "\t\t\t<li><a epub:type=\"toc\" href=\"nav.xhtml\">目次</a></li>\n".as_bytes(),
        )?;

        for chapter in &self.vzip.nresult.xhtmls.chapters {
            let filename = format!("xhtml/sec{:>04}.xhtml", chapter.xhtml_id);
            let id = chapter.get_id();
            let name = &chapter.name;
            writeln!(
                writer,
                "\t\t\t<li><a epub:type=\"bodymatter\" href=\"{}#{}\">{}</a></li>",
                filename, id, name
            )?;
        }

        writer.write_all(b"\t\t</ol>\n")?;
        writer.write_all(b"\t</nav>\n")?;
        Ok(())
    }

    fn write_nav_toc(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(b"\t<nav epub:type=\"toc\" id=\"toc\">\n")?;
        writer.write_all("\t\t<h1>目　次</h1>\n".as_bytes())?;
        writer.write_all(b"\t\t<ol>\n")?;

        for chapter in &self.vzip.nresult.xhtmls.chapters {
            let filename = format!("xhtml/sec{:>04}.xhtml", chapter.xhtml_id);
            let id = chapter.get_id();
            let name = &chapter.name;
            writeln!(
                writer,
                "\t\t\t<li><a href=\"{}#{}\">{}</a></li>",
                filename, id, name
            )?;
        }

        writer.write_all(b"\t\t</ol>\n")?;
        writer.write_all(b"\t</nav>\n")?;
        Ok(())
    }

    pub fn write_nav(&self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(include_bytes!("../../assets/nav_header"))?;

        self.write_nav_head(writer)?;

        writer.write_all(b"<body>\n")?;
        self.write_nav_landmarks(writer)?;
        self.write_nav_toc(writer)?;
        writer.write_all(b"</body>\n")?;
        writer.write_all(b"</html>")?;

        Ok(())
    }
}
