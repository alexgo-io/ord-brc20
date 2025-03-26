use super::*;

#[allow(unused)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(C)]
pub enum BrotliEncoderMode {
  BrotliModeGeneric = 0,
  BrotliModeText = 1,
  BrotliModeFont = 2,
  BrotliForceLsbPrior = 3,
  BrotliForceMsbPrior = 4,
  BrotliForceUtf8Prior = 5,
  BrotliForceSignedPrior = 6,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Media {
  Audio,
  Code(Language),
  Font,
  Iframe,
  Image,
  Markdown,
  Model,
  Pdf,
  Text,
  Unknown,
  Video,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Language {
  Css,
  JavaScript,
  Json,
  Python,
  Yaml,
}

impl Display for Language {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Css => "css",
        Self::JavaScript => "javascript",
        Self::Json => "json",
        Self::Python => "python",
        Self::Yaml => "yaml",
      }
    )
  }
}

impl Media {
  #[rustfmt::skip]
  const TABLE: &'static [(&'static str, BrotliEncoderMode, Media, &'static [&'static str])] = &[
    ("application/cbor",            BrotliEncoderMode::BrotliModeGeneric, Media::Unknown,                    &["cbor"]),
    ("application/json",            BrotliEncoderMode::BrotliModeText,    Media::Code(Language::Json),       &["json"]),
    ("application/octet-stream",    BrotliEncoderMode::BrotliModeGeneric, Media::Unknown,                    &["bin"]),
    ("application/pdf",             BrotliEncoderMode::BrotliModeGeneric, Media::Pdf,                        &["pdf"]),
    ("application/pgp-signature",   BrotliEncoderMode::BrotliModeText,    Media::Text,                       &["asc"]),
    ("application/protobuf",        BrotliEncoderMode::BrotliModeGeneric, Media::Unknown,                    &["binpb"]),
    ("application/x-javascript",    BrotliEncoderMode::BrotliModeText,    Media::Code(Language::JavaScript), &[]),
    ("application/yaml",            BrotliEncoderMode::BrotliModeText,    Media::Code(Language::Yaml),       &["yaml", "yml"]),
    ("audio/flac",                  BrotliEncoderMode::BrotliModeGeneric, Media::Audio,                      &["flac"]),
    ("audio/mpeg",                  BrotliEncoderMode::BrotliModeGeneric, Media::Audio,                      &["mp3"]),
    ("audio/wav",                   BrotliEncoderMode::BrotliModeGeneric, Media::Audio,                      &["wav"]),
    ("font/otf",                    BrotliEncoderMode::BrotliModeGeneric, Media::Font,                       &["otf"]),
    ("font/ttf",                    BrotliEncoderMode::BrotliModeGeneric, Media::Font,                       &["ttf"]),
    ("font/woff",                   BrotliEncoderMode::BrotliModeGeneric, Media::Font,                       &["woff"]),
    ("font/woff2",                  BrotliEncoderMode::BrotliModeFont,    Media::Font,                       &["woff2"]),
    ("image/apng",                  BrotliEncoderMode::BrotliModeGeneric, Media::Image,                      &["apng"]),
    ("image/avif",                  BrotliEncoderMode::BrotliModeGeneric, Media::Image,                      &[]),
    ("image/gif",                   BrotliEncoderMode::BrotliModeGeneric, Media::Image,                      &["gif"]),
    ("image/jpeg",                  BrotliEncoderMode::BrotliModeGeneric, Media::Image,                      &["jpg", "jpeg"]),
    ("image/png",                   BrotliEncoderMode::BrotliModeGeneric, Media::Image,                      &["png"]),
    ("image/svg+xml",               BrotliEncoderMode::BrotliModeText,    Media::Iframe,                     &["svg"]),
    ("image/webp",                  BrotliEncoderMode::BrotliModeGeneric, Media::Image,                      &["webp"]),
    ("model/gltf+json",             BrotliEncoderMode::BrotliModeText,    Media::Model,                      &["gltf"]),
    ("model/gltf-binary",           BrotliEncoderMode::BrotliModeGeneric, Media::Model,                      &["glb"]),
    ("model/stl",                   BrotliEncoderMode::BrotliModeGeneric, Media::Unknown,                    &["stl"]),
    ("text/css",                    BrotliEncoderMode::BrotliModeText,    Media::Code(Language::Css),        &["css"]),
    ("text/html",                   BrotliEncoderMode::BrotliModeText,    Media::Iframe,                     &[]),
    ("text/html;charset=utf-8",     BrotliEncoderMode::BrotliModeText,    Media::Iframe,                     &["html"]),
    ("text/javascript",             BrotliEncoderMode::BrotliModeText,    Media::Code(Language::JavaScript), &["js"]),
    ("text/markdown",               BrotliEncoderMode::BrotliModeText,    Media::Markdown,                   &[]),
    ("text/markdown;charset=utf-8", BrotliEncoderMode::BrotliModeText,    Media::Markdown,                   &["md"]),
    ("text/plain",                  BrotliEncoderMode::BrotliModeText,    Media::Text,                       &[]),
    ("text/plain;charset=utf-8",    BrotliEncoderMode::BrotliModeText,    Media::Text,                       &["txt"]),
    ("text/x-python",               BrotliEncoderMode::BrotliModeText,    Media::Code(Language::Python),     &["py"]),
    ("video/mp4",                   BrotliEncoderMode::BrotliModeGeneric, Media::Video,                      &["mp4"]),
    ("video/webm",                  BrotliEncoderMode::BrotliModeGeneric, Media::Video,                      &["webm"]),
  ];
}

impl FromStr for Media {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for entry in Self::TABLE {
      if entry.0 == s {
        return Ok(entry.2);
      }
    }

    Err(anyhow!("unknown content type: {s}"))
  }
}
