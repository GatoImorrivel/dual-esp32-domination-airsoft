use esp_idf_svc::http::server::{EspHttpConnection, Request};

pub mod auth;
pub trait Middleware<R> {
    fn handle(&self, req: &mut Request<&mut EspHttpConnection>) -> anyhow::Result<R>;
}
