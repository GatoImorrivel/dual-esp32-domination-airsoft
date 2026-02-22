use serde::Serialize;

pub mod routes;
pub mod server;

pub enum ResponseBody {
    String(String),
    StaticString(&'static str),
    Bytes(&'static [u8]),
}

pub struct Response {
    status_code: u16,
    content_type: ContentType,
    body: ResponseBody,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            body: ResponseBody::StaticString(""),
            content_type: ContentType::Text,
            status_code: 200,
        }
    }

    pub fn body(&self) -> &[u8] {
        match &self.body {
            ResponseBody::StaticString(payload) => payload.as_bytes(),
            ResponseBody::String(payload) => payload.as_bytes(),
            ResponseBody::Bytes(payload) => payload,
        }
    }
}

pub struct Json(String);

impl Json {
    pub fn new<T: Serialize + ?Sized>(data: &T) -> anyhow::Result<Self> {
        Ok(Self {
            0: serde_json::to_string(data)?,
        })
    }
}

impl Into<Response> for Json {
    fn into(self) -> Response {
        Response {
            status_code: 200,
            content_type: ContentType::Json,
            body: ResponseBody::String(self.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MediaType(&'static str);

#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    Js,
    Css,
    Html,
    Svg,
    Png,
    Jpg,
    Ico,
    Woff,
    Woff2,
    Ttf,
    Json,
    OctetStream,
    Text,
}

impl ContentType {
    pub fn from_file_extension<S: AsRef<str>>(extension: S) -> Self {
        match extension.as_ref() {
            "js" => Self::Js,
            "mjs" => Self::Js,
            "css" => Self::Css,
            "html" => Self::Html,
            "svg" => Self::Svg,
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpg,
            "ico" => Self::Ico,
            "woff" => Self::Woff,
            "woff2" => Self::Woff2,
            "ttf" => Self::Ttf,
            "json" => Self::Json,
            "txt" => Self::Text,
            _ => Self::OctetStream,
        }
    }

    pub fn into_media_type(&self) -> MediaType {
        let media_type = match self {
            Self::Js => "application/javascript",
            Self::Css => "text/css",
            Self::Html => "text/html",
            Self::Svg => "image/svg+xml",
            Self::Png => "image/png",
            Self::Jpg => "image/jpeg",
            Self::Ico => "image/x-icon",
            Self::Woff => "font/woff",
            Self::Woff2 => "font/woff2",
            Self::Ttf => "font/ttf",
            Self::Json => "application/json",
            Self::OctetStream => "application/octet-stream",
            Self::Text => "text/plain",
        };
        MediaType(media_type)
    }
}

impl<S: AsRef<str>> From<S> for ContentType {
    fn from(value: S) -> Self {
        Self::from_file_extension(value)
    }
}

impl Into<MediaType> for ContentType {
    fn into(self) -> MediaType {
        self.into_media_type()
    }
}

impl Into<&'static str> for ContentType {
    fn into(self) -> &'static str {
        self.into_media_type().0
    }
}
