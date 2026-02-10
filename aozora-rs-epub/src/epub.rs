use std::io::Seek;

use crate::{
    opf::EpubMeta,
    zip::{AozoraZip, ImgExtension},
};
use aozora_rs_core::{AZResult, AZResultC};
use aozora_rs_xhtml::get_xhtml_filename;
use miette::Diagnostic;
use std::io::Write;
use thiserror::Error;
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Diagnostic, Debug, Error)]
#[error("依存関係にあるファイルが存在しません")]
#[diagnostic(
    code(aozora_rs_epub::dependency_no_found),
    help("対象のファイルが存在し、プログラムがアクセスできる状態にあることを確認してください。")
)]
struct DependencyNotFound {
    #[source_code]
    target: String,
}

pub fn from_aozora_zip<T>(
    acc: impl Write + Seek,
    azz: AozoraZip,
    styles: Vec<&str>,
) -> Result<AZResult<()>, Box<dyn std::error::Error>> {
    let mut writer = ZipWriter::new(acc);
    let options = SimpleFileOptions::default();
    let meta = EpubMeta::new(
        azz.nresult.title.as_str(),
        azz.nresult.author.as_str(),
        "ja",
        styles,
        azz.nresult
            .xhtmls
            .dependency
            .iter()
            .filter_map(|img| {
                let ext = match img.as_str() {
                    "png" | "PNG" => Some(ImgExtension::Png),
                    "jpg" | "JPG" | "jpeg" | "JPEG" => Some(ImgExtension::Jpeg),
                    "gif" | "GIF" => Some(ImgExtension::Gif),
                    "svg" | "SVG" => Some(ImgExtension::Svg),
                    _ => None,
                }?;
                Some((img.clone(), ext))
            })
            .collect::<Vec<(String, ImgExtension)>>(),
        azz.nresult.xhtmls,
    );

    writer.start_file("mimetype", options)?;
    writer.write_all(include_str!("../assets/mimetype").as_bytes())?;

    writer.start_file("META-INF/container.xml", options)?;
    writer.write_all(include_str!("../assets/container.xml").as_bytes())?;

    writer.start_file("item/standard.opf", options)?;
    writer.write_all(meta.into_opf().as_bytes())?;

    writer.start_file("item/toc.ncx", options)?;
    writer.write_all(meta.into_ncx().as_bytes())?;

    writer.start_file("item/nav.xhtml", options)?;
    writer.write_all(meta.into_nav().as_bytes())?;

    for (i, x) in meta.xhtmls.xhtmls.iter().enumerate() {
        writer.start_file(format!("item/xhtml/{}", get_xhtml_filename(i)), options)?;
        writer.write_all(
            include_str!("../assets/template.xhtml")
                .replace("［＃コンテンツ］", x)
                .replace("［＃タイトル］", meta.title)
                .as_bytes(),
        )?;
    }

    let mut azresult = AZResultC::new();
    for d in meta.xhtmls.dependency {
        if let Some(img) = azz.images.get(&d) {
            writer.start_file(format!("item/image/{}", d), options)?;
            writer.write(&img.1)?;
        } else {
            azresult.push(
                DependencyNotFound {
                    target: d.to_string(),
                }
                .into(),
            );
        }
    }

    Ok(azresult.finally(()))
}
