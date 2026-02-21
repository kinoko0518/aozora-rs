use std::io::Write;

use crate::epub::EpubWriter;

impl EpubWriter<'_> {
    fn write_ncx_head(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all("<head>".as_bytes())?;

        writeln!(
            writer,
            "\t<meta name=\"dtb:uid\" content=\"urn:uuid:{}\"/>",
            self.uuid()
        )?;
        writer.write_all("\t<meta name=\"dtb:depth\" content=\"1\"/>".as_bytes())?;
        writer.write_all("\t<meta name=\"dtb:totalPageCount\" content=\"0\"/>".as_bytes())?;
        writer.write_all("\t<meta name=\"dtb:maxPageNumber\" content=\"0\"/>".as_bytes())?;

        writer.write_all("</head>".as_bytes())?;
        Ok(())
    }

    fn write_ncx_navmaps(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all("<navMap>\n".as_bytes())?;

        let iter = self
            .nresult
            .xhtmls
            .chapters
            .iter()
            .map(|c| (&c.name, c.xhtml_id));
        for c in iter {
            writer.write_all("\t<navPoint id=\"toc1\" playOrder=\"1\">\n".as_bytes())?;
            writer.write_all("\t\t<navLabel>\n".as_bytes())?;
            writeln!(writer, "\t\t\t<text>{}</text>", c.0)?;
            writer.write_all("\t\t</navLabel>\n".as_bytes())?;
            writeln!(writer, "\t\t<content src=\"xhtml/sec{:>04}\"/>", c.1)?;
            writer.write_all("\t</navPoint>\n".as_bytes())?;
        }

        writer.write_all("</navMap>".as_bytes())?;
        Ok(())
    }

    pub fn write_ncx(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".as_bytes())?;
        writer.write_all(
            "<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\">\n".as_bytes(),
        )?;

        self.write_ncx_head(writer)?;
        writeln!(
            writer,
            "\n<docTitle>\n\t<text>{}</text>\n</docTitle>",
            self.nresult.meta.title
        )?;
        self.write_ncx_navmaps(writer)?;

        writer.write_all("</ncx>".as_bytes())?;

        Ok(())
    }
}
