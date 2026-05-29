use esp_idf_svc::{
    http::{
        headers::content_type,
        server::{EspHttpConnection, EspHttpServer, Request},
    },
    io::Read,
};

use crate::{
    http::{write_all_embedded, ContentType, RequestContext, Response as AppResponse},
    middleware::auth::client_ipv4,
};

/// Login JSON body can exceed 128 bytes (username + base64url SHA-512 hash).
const MAX_PAYLOAD_LEN: usize = 512;

#[cfg(debug_assertions)]
fn json_response_headers(content_type_value: &'static str) -> [(&'static str, &'static str); 4] {
    [
        content_type(content_type_value),
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Authorization"),
    ]
}

#[cfg(not(debug_assertions))]
fn json_response_headers(content_type_value: &'static str) -> [(&'static str, &'static str); 1] {
    [content_type(content_type_value)]
}

#[cfg(debug_assertions)]
fn options_headers() -> [(&'static str, &'static str); 3] {
    [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Authorization"),
    ]
}

fn send_response(
    request: Request<&mut EspHttpConnection>,
    response: AppResponse,
) -> anyhow::Result<()> {
    let status = response.status_code;
    let content_type_value = response.content_type.into_media_type().0;
    let headers = json_response_headers(content_type_value);
    let mut writer = request.into_response(status, None, &headers)?;
    response
        .write_body(&mut writer)
        .map_err(|e| anyhow::anyhow!("response write failed: {:?}", e))?;
    Ok(())
}

fn auth_failure_status(message: &str) -> u16 {
    if message.contains("autorizado") || message.contains("invalido") {
        401
    } else {
        500
    }
}

fn send_error(
    request: Request<&mut EspHttpConnection>,
    status: u16,
    message: &'static str,
) -> anyhow::Result<()> {
    let headers = json_response_headers(ContentType::Text.into_media_type().0);
    let mut writer = request.into_response(status, None, &headers)?;
    write_all_embedded(&mut writer, message.as_bytes())
        .map_err(|e| anyhow::anyhow!("error response write failed: {:?}", e))?;
    Ok(())
}

pub struct HttpServer {
    esp_http_server: EspHttpServer<'static>,
}

impl HttpServer {
    pub fn new() -> Self {
        let mut server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration {
            stack_size: 32768,
            max_uri_handlers: 32,
            ..Default::default()
        })
        .unwrap();

        #[cfg(debug_assertions)]
        {
            server
                .fn_handler("/*", esp_idf_svc::http::Method::Options, |request| {
                    let mut writer = request.into_response(204, None, &options_headers())?;
                    write_all_embedded(&mut writer, &[])
                        .map_err(|e| anyhow::anyhow!("options response: {:?}", e))
                })
                .unwrap();
        }

        Self {
            esp_http_server: server,
        }
    }

    pub fn get<S, F>(&mut self, url: S, handler: F) -> &mut Self
    where
        F: Fn(&mut Request<&mut EspHttpConnection>, &RequestContext) -> anyhow::Result<AppResponse>
            + Send
            + Sync
            + 'static,
        S: AsRef<str>,
    {
        self.esp_http_server
            .fn_handler(
                url.as_ref(),
                esp_idf_svc::http::Method::Get,
                move |mut request| {
                    let ctx = RequestContext {
                        client_ip: client_ipv4(&mut request).ok(),
                        authorization: request.header("Authorization").map(str::to_string),
                    };
                    log::info!("HTTP GET {} ip={:?}", request.uri(), ctx.client_ip);

                    match handler(&mut request, &ctx) {
                        Ok(response) => send_response(request, response),
                        Err(err) => {
                            log::error!("Error handling GET {}: {}", request.uri(), err);
                            let status = auth_failure_status(&err.to_string());
                            if status == 401 {
                                send_error(request, 401, "Nao autorizado")
                            } else {
                                send_error(request, 500, "Erro interno")
                            }
                        }
                    }
                },
            )
            .unwrap();

        self
    }

    pub fn post<S, B, F>(&mut self, url: S, handler: F) -> &mut Self
    where
        S: AsRef<str>,
        B: for<'a> serde::Deserialize<'a> + 'static,
        F: Fn(B, &mut Request<&mut EspHttpConnection>, &RequestContext) -> anyhow::Result<AppResponse>
            + Send
            + Sync
            + 'static,
    {
        self.esp_http_server
            .fn_handler(
                url.as_ref(),
                esp_idf_svc::http::Method::Post,
                move |mut request| {
                    let ctx = RequestContext {
                        client_ip: client_ipv4(&mut request).ok(),
                        authorization: request.header("Authorization").map(str::to_string),
                    };
                    log::info!("HTTP POST {} ip={:?}", request.uri(), ctx.client_ip);

                    let len = match request
                        .header("Content-Length")
                        .unwrap_or("0")
                        .parse::<usize>()
                    {
                        Ok(len) => len,
                        Err(_) => return send_error(request, 500, "Erro interno"),
                    };

                    if len > MAX_PAYLOAD_LEN {
                        return send_error(request, 413, "Request too big");
                    }

                    let mut buf = vec![0u8; len];
                    if request.read_exact(&mut buf).is_err() {
                        return send_error(request, 400, "Leitura invalida");
                    }

                    let data = match serde_json::from_slice::<B>(&buf) {
                        Ok(data) => data,
                        Err(_) => return send_error(request, 422, "JSON invalido"),
                    };

                    match handler(data, &mut request, &ctx) {
                        Ok(response) => send_response(request, response),
                        Err(err) => {
                            log::error!("Error handling POST {}: {}", request.uri(), err);
                            let status = auth_failure_status(&err.to_string());
                            if status == 401 {
                                send_error(request, 401, "Nao autorizado")
                            } else {
                                send_error(request, 500, "Erro interno")
                            }
                        }
                    }
                },
            )
            .unwrap();

        self
    }
}
