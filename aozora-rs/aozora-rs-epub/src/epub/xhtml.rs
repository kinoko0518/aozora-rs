use std::io::Write;

use crate::epub::EpubWriter;

impl EpubWriter<'_> {
    pub(crate) fn write_xhtml(
        &self,
        content: &str,
        writer: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        writer.write_all(include_bytes!("../../assets/xhtml_header"))?;

        writer.write_all(b"<head>\n\t<meta charset=\"UTF-8\" />\n")?;

        writeln!(writer, "\t<title>{}</title>", &self.meta.title)?;
        self.apply_css(writer, "../style/")?;

        writer.write_all(b"</head>\n<body>\n\t<div class=\"main\">\n")?;
        writer.write_all(content.as_bytes())?;
        writer.write_all(b"\n\t</div>\n</body>\n</html>\n")?;

        Ok(())
    }

    pub(crate) fn write_injected_page<H>(
        &self,
        writer: &mut impl Write,
        hyle: &H,
        injector: &dyn Fn(&mut dyn Write, &H) -> std::io::Result<()>,
    ) -> Result<(), std::io::Error> {
        writer.write_all(include_bytes!("../../assets/xhtml_header"))?;
        writer.write_all(b"<head>\n\t<meta charset=\"UTF-8\" />\n")?;
        writeln!(writer, "\t<title>{}</title>", &self.meta.title)?;
        self.apply_css(writer, "../style/")?;
        writer.write_all(b"</head>\n<body>\n\t<div class=\"main\">\n")?;
        injector(writer, hyle)?;
        writer.write_all(b"\n\t</div>\n</body>\n</html>\n")?;
        Ok(())
    }
}
