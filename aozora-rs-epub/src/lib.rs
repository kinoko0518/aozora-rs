mod epub;
mod zip;

pub use epub::{EpubSetting, from_aozora_zip, from_sjis_aozora_zip, from_utf8_aozora_zip};
pub use zip::{AozoraZip, AozoraZipError};

pub enum ImgExtension {
    Png,
    Jpeg,
    Gif,
    Svg,
}

impl ImgExtension {
    pub fn into_media_type(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Gif => "gif",
            Self::Svg => "svg+xml",
        }
    }

    pub fn from_extension(from: &str) -> Option<Self> {
        match from {
            "jpg" | "jpeg" | "JPG" | "JPEG" => Some(Self::Jpeg),
            "png" | "PNG" => Some(Self::Png),
            "gif" | "GIF" => Some(Self::Gif),
            "svg" | "SVG" => Some(Self::Svg),
            _ => None,
        }
    }
}
