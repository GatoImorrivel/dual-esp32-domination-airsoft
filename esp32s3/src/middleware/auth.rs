use std::{
    env,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use base64::Engine;

use anyhow::Context;

use base64::engine::general_purpose;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::middleware::Middleware;

type HmacSha256 = Hmac<Sha256>;
const SECRET: &str = env!("HMAC_SECRET");

#[derive(Debug, PartialEq, Eq)]
pub struct AuthToken {
    hash: String,
    expires_at: u64, // Unix Timestamap
    ip: String,
}

impl AuthToken {
    pub fn into_string(self, secret: &[u8]) -> String {
        let payload = format!("{}.{}.{}", self.hash, self.expires_at, self.ip);

        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(payload.as_bytes());
        let signature = mac.finalize().into_bytes();

        let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload);
        let sig_b64 = general_purpose::URL_SAFE_NO_PAD.encode(signature);

        format!("{}.{}", payload_b64, sig_b64)
    }

    pub fn from_string(token: &str, secret: &[u8]) -> Option<Self> {
        let mut parts = token.split('.');
        let payload_b64 = parts.next()?;
        let sig_b64 = parts.next()?;

        if parts.next().is_some() {
            return None;
        }

        let payload_bytes = general_purpose::URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
        let payload = String::from_utf8(payload_bytes).ok()?;

        let sig_bytes = general_purpose::URL_SAFE_NO_PAD.decode(sig_b64).ok()?;

        let mut mac = HmacSha256::new_from_slice(secret).ok()?;
        mac.update(payload.as_bytes());

        mac.verify_slice(&sig_bytes).ok()?;

        let mut payload_parts = payload.split('.');
        let hash = payload_parts.next()?.to_string();
        let expires_at = payload_parts.next()?.parse::<u64>().ok()?;
        let ip = payload_parts.next()?.to_string();

        if payload_parts.next().is_some() {
            return None;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

        if now > expires_at {
            return None;
        }

        Some(AuthToken {
            hash,
            expires_at,
            ip,
        })
    }
}

impl Into<String> for AuthToken {
    fn into(self) -> String {
        self.into_string(SECRET.as_bytes())
    }
}

impl TryFrom<String> for AuthToken {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let token = AuthToken::from_string(&value, SECRET.as_bytes());
        if let Some(token) = token {
            return Ok(token);
        }

        Err("Token invalido")
    }
}

impl TryFrom<&str> for AuthToken {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let token = AuthToken::from_string(value, SECRET.as_bytes());
        if let Some(token) = token {
            return Ok(token);
        }

        Err("Token invalido")
    }
}

pub struct AdminUser {
    username: String,
    password: String,
    nonce: Option<String>,
    // Exact time the nonce expires
    nonce_expiration_instant: Option<Instant>,
    current_token: Option<AuthToken>,
}

pub struct AuthMiddleware {
    admins: [AdminUser; 1],
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self {
            admins: [AdminUser {
                username: env!("ADMIN_USER").to_string(),
                password: env!("ADMIN_PASS").to_string(),
                nonce_expiration_instant: None,
                nonce: None,
                current_token: None,
            }],
        }
    }
}

impl Middleware<()> for AuthMiddleware {
    fn handle(
        &self,
        req: &mut esp_idf_svc::http::server::Request<
            &mut esp_idf_svc::http::server::EspHttpConnection,
        >,
    ) -> anyhow::Result<()> {
        let token = req
            .header("Authorization")
            .ok_or_else(|| anyhow::anyhow!("Não autorizado"))?;

        let parsed_token = AuthToken::try_from(token).map_err(|err| anyhow::anyhow!(err))?;

        let ip = req
            .connection()
            .raw_connection()
            .context("Falha de autenticacao")?
            .source_ipv4()
            .map_err(|_| anyhow::anyhow!("Falha de autenticacao"))?;

        if parsed_token.ip != ip.to_string() {
            return Err(anyhow::anyhow!("Não autorizado"));
        }

        let matching_user = self.admins.iter().find(|admin| {
            admin
                .current_token
                .as_ref()
                .map_or(false, |t| t == &parsed_token)
        });

        if matching_user.is_some() {
            return Ok(());
        }

        Err(anyhow::anyhow!("Não autorizado"))
    }
}
