use esp_idf_svc::{
    http::{headers::content_type, server::EspHttpServer},
    io::{Read, Write},
};

use crate::http::{ContentType, Response};

const MAX_PAYLOAD_LEN: usize = 128;

pub struct HttpServer {
    esp_http_server: EspHttpServer<'static>,
}

impl HttpServer {
    pub fn new() -> Self {
        let mut server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration {
            stack_size: 12288,
            ..Default::default()
        })
        .unwrap();

        #[cfg(debug_assertions)]
        {
            server
                .fn_handler("/*", esp_idf_svc::http::Method::Options, |request| {
                    request
                        .into_response(
                            204,
                            None,
                            &[
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Origin", "*"),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Headers", "Content-Type"),
                            ],
                        )?
                        .write_all(&[])
                })
                .unwrap();
        }

        Self {
            esp_http_server: server,
        }
    }
    pub fn get<S: AsRef<str>, F: Fn() -> anyhow::Result<Response> + Send + Sync + 'static>(
        &mut self,
        url: S,
        handler: F,
    ) -> &mut Self {
        self.esp_http_server
            .fn_handler(
                url.as_ref(),
                esp_idf_svc::http::Method::Get,
                move |request| {
                    let response = handler();
                    if let Err(err) = response {
                        log::error!("Error handling {}: {}", request.uri(), err);
                        return request
                            .into_response(
                                500,
                                None,
                                &[
                                    content_type(ContentType::Text.into_media_type().0),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Origin", "*"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Headers", "Content-Type"),
                                ],
                            )?
                            .write_all(err.to_string().as_bytes());
                    }
                    let response = response.unwrap();
                    let body = response.body();
                    request
                        .into_response(
                            response.status_code,
                            None,
                            &[
                                content_type(response.content_type.into_media_type().0),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Origin", "*"),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Headers", "Content-Type"),
                            ],
                        )?
                        .write_all(body)
                        .map(|_| ())
                },
            )
            .unwrap();

        self
    }

    pub fn post<
        S: AsRef<str>,
        B: for<'a> serde::Deserialize<'a> + 'static,
        F: Fn(B) -> anyhow::Result<Response> + Send + Sync + 'static,
    >(
        &mut self,
        url: S,
        handler: F,
    ) -> &mut Self {
        self.esp_http_server
            .fn_handler(
                url.as_ref(),
                esp_idf_svc::http::Method::Post,
                move |mut request| {
                    let len = request
                        .header("Content-Length")
                        .unwrap_or("0")
                        .parse::<usize>();

                    if let Err(err) = len {
                        return request
                            .into_response(
                                500,
                                None,
                                &[
                                    content_type(ContentType::Text.into_media_type().0),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Origin", "*"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Headers", "Content-Type"),
                                ],
                            )?
                            .write_all(err.to_string().as_bytes());
                    }

                    let len = len.unwrap();

                    if len > MAX_PAYLOAD_LEN {
                        return request
                            .into_response(
                                413,
                                None,
                                &[
                                    content_type(ContentType::Text.into_media_type().0),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Origin", "*"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Headers", "Content-Type"),
                                ],
                            )?
                            .write_all("Request too big".as_bytes());
                    }

                    let mut buf = vec![0; len];
                    let read_result = request.read_exact(&mut buf);

                    if let Err(err) = read_result {
                        return request
                            .into_response(
                                400,
                                None,
                                &[
                                    content_type(ContentType::Text.into_media_type().0),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Origin", "*"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Headers", "Content-Type"),
                                ],
                            )?
                            .write_all(err.to_string().as_bytes());
                    }

                    let data = serde_json::from_slice::<B>(&buf);

                    if let Err(err) = data {
                        return request
                            .into_response(
                                422,
                                None,
                                &[
                                    content_type(ContentType::Text.into_media_type().0),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Origin", "*"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Headers", "Content-Type"),
                                ],
                            )?
                            .write_all(err.to_string().as_bytes());
                    }

                    let response = handler(data.unwrap());
                    if let Err(err) = response {
                        log::error!("Error handling {}: {}", request.uri(), err);
                        return request
                            .into_response(
                                500,
                                None,
                                &[
                                    content_type(ContentType::Text.into_media_type().0),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Origin", "*"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                    #[cfg(debug_assertions)]
                                    ("Access-Control-Allow-Headers", "Content-Type"),
                                ],
                            )?
                            .write_all(err.to_string().as_bytes());
                    }
                    let response = response.unwrap();
                    request
                        .into_response(
                            response.status_code,
                            None,
                            &[
                                content_type(response.content_type.into_media_type().0),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Origin", "*"),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                                #[cfg(debug_assertions)]
                                ("Access-Control-Allow-Headers", "Content-Type"),
                            ],
                        )?
                        .write_all(response.body())
                },
            )
            .unwrap();

        self
    }
}
