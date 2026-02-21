use std::io::Write;

use crate::epub::EpubWriter;

impl EpubWriter<'_> {
    pub fn write_xhtml(
        &self,
        content: &str,
        writer: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        writer.write_all(include_bytes!("../../assets/xhtml_header"))?;

        writer.write_all(b"<head>\n\t<meta charset=\"UTF-8\" />\n")?;

        writeln!(writer, "\t<title>{}</title>", &self.nresult.meta.title)?;
        self.apply_css(writer, "../style/")?;

        writer.write_all(b"</head>\n<body>\n\t<div class=\"main\">\n")?;
        writer.write_all(content.as_bytes())?;
        writer.write_all(b"\n\t</div>\n</body>\n</html>\n")?;

        Ok(())
    }
}
