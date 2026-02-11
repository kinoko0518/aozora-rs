use std::io::Write;

use crate::epub::EpubWriter;

impl EpubWriter<'_> {
    fn write_opf_metadata(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer
            .write_all("\t<metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n".as_bytes())?;
        write!(
            writer,
            "\t\t<!-- 作品名 -->\n\t\t<dc:title id=\"title\">{}</dc:title>\n",
            &self.vzip.nresult.title
        )?;
        write!(
            writer,
            "\t\t<!-- 著者名 -->\n\t\t<dc:creator id=\"creator01\">{}</dc:creator>\n",
            &self.vzip.nresult.author
        )?;
        write!(
            writer,
            "\t\t<!-- 言語 -->\n\t\t<dc:language id=\"pub-lang\">{}</dc:language>\n",
            self.setting.language
        )?;
        write!(
            writer,
            "\t\t<!-- ファイルid -->\n\t\t<dc:identifier id=\"unique-id\">urn:uuid:{}</dc:identifier>\n",
            self.uuid()
        )?;
        write!(
            writer,
            "\t\t<!-- 更新日 -->\n\t\t<meta property=\"dcterms:modified\">{}</meta>\n",
            self.lud.format("%Y-%m-%dT%H:%M:%SZ")
        )?;
        writer.write_all("\t\t<!-- etc. -->\n".as_bytes())?;
        writer.write_all("\t\t<meta property=\"ebpaj:guide-version\">1.1.3</meta>\n".as_bytes())?;
        writer.write_all("\t\t<meta property=\"ibooks:version\">1.1.2</meta>\n".as_bytes())?;
        writer.write_all("\t</metadata>".as_bytes())?;
        Ok(())
    }

    fn write_opf_manifest(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all("\t<manifest>\n\t\t<!-- navigation -->\n".as_bytes())?;

        // nav.xhtmlを宣言
        writer.write_all(
            "\t\t<item\n\t\t\tmedia-type=\"application/xhtml+xml\"\n\t\t\tid=\"nav\"\n".as_bytes(),
        )?;
        writer
            .write_all("\t\t\thref=\"nav.xhtml\"\n\t\t\tproperties=\"nav\"\n\t\t/>\n".as_bytes())?;

        // toc.ncxを宣言
        writer.write_all(
            "\t\t<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\" />\n"
                .as_bytes(),
        )?;

        // styleを宣言
        writer.write_all("\t\t<!-- style -->\n".as_bytes())?;
        for (id, s) in self.css().enumerate() {
            write!(
                writer,
                "\t\t<item id=\"style{:>04}\" href=\"{}\" media-type=\"text/css\"/>\n",
                id, s
            )?;
        }

        // imageを宣言
        writer.write_all("\t\t<!-- image -->\n".as_bytes())?;
        for (id, (path, ext)) in self.images().enumerate() {
            write!(
                writer,
                "\t\t<item id=\"image{:>04}\" href=\"{}\" media-type=\"image/{}\"/>\n",
                id,
                path,
                ext.into_media_type()
            )?;
        }

        // XHTMLを宣言
        writer.write_all("\t\t<!-- xhtml -->\n".as_bytes())?;
        for (id, xhtml) in self.xhtmls().enumerate() {
            write!(
                writer,
                "\t\t<item id=\"sec{:>04}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
                id, xhtml
            )?;
        }

        // manifestを終了
        writer.write_all("\t</manifest>".as_bytes())?;

        Ok(())
    }

    fn write_opf_spine(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        let spines = self
            .vzip
            .nresult
            .xhtmls
            .xhtmls
            .iter()
            .enumerate()
            .map(|(num, _)| num);

        writeln!(
            writer,
            "\t<spine page-progression-direction=\"{}\" toc=\"ncx\">",
            if self.setting.is_rtl { "rtl" } else { "ltl" }
        )?;
        writer.write_all("\t\t<itemref idref=\"nav\" linear=\"yes\" />\n".as_bytes())?;
        for s in spines {
            writeln!(
                writer,
                "\t\t<itemref linear=\"yes\" idref=\"sec{:>04}\" />",
                s
            )?;
        }
        writer.write_all("</spine>".as_bytes())?;
        Ok(())
    }

    pub fn write_opf(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all(include_bytes!("../../assets/opf_header"))?;

        self.write_opf_metadata(writer)?;
        writer.write_all("\n\n".as_bytes())?;
        self.write_opf_manifest(writer)?;
        writer.write_all("\n\n".as_bytes())?;
        self.write_opf_spine(writer)?;
        writer.write_all("\n\n".as_bytes())?;

        writer.write_all("</package>".as_bytes())?;

        Ok(())
    }
}
