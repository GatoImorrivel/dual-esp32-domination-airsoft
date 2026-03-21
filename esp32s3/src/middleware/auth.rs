use std::{
    env,
    sync::{Arc, OnceLock, RwLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use base64::Engine;

use anyhow::Context;

use base64::engine::general_purpose;
use esp_idf_svc::sys::esp_random;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;
const SECRET: &str = env!("HMAC_SECRET");

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthToken {
    hash: String,
    expires_at: u64, // Unix Timestamap
    ip: String,
}

impl AuthToken {
    pub fn new(ip: String, ttl_secs: u64) -> Self {
        let mut buf = [0u8; 16];
        for chunk in buf.chunks_mut(4) {
            let r = unsafe { esp_random() };
            chunk.copy_from_slice(&r.to_le_bytes());
        }

        let hash = general_purpose::URL_SAFE_NO_PAD.encode(buf);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            hash,
            ip,
            expires_at: now + ttl_secs,
        }
    }

    pub fn into_string(self) -> String {
        let secret = SECRET.as_bytes();
        let payload = format!("{}.{}.{}", self.hash, self.expires_at, self.ip);

        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(payload.as_bytes());
        let signature = mac.finalize().into_bytes();

        let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload);
        let sig_b64 = general_purpose::URL_SAFE_NO_PAD.encode(signature);

        format!("{}.{}", payload_b64, sig_b64)
    }

    pub fn from_string(token: &str) -> Option<Self> {
        let secret = SECRET.as_bytes();
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
        self.into_string()
    }
}

impl TryFrom<String> for AuthToken {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let token = AuthToken::from_string(&value);
        if let Some(token) = token {
            return Ok(token);
        }

        Err("Token invalido")
    }
}

impl TryFrom<&str> for AuthToken {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let token = AuthToken::from_string(value);
        if let Some(token) = token {
            return Ok(token);
        }

        Err("Token invalido")
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    username: String,
    password: String,
    nonce: Option<String>,
    nonce_generation_instant: Option<Instant>,
    current_token: Option<AuthToken>,
}

static USER_MANAGER: OnceLock<UserManager> = OnceLock::new();

fn load_admins_users() -> [AdminUser; 1] {
    [AdminUser {
        username: env!("ADMIN_USER").to_string(),
        password: env!("ADMIN_PASS").to_string(),
        nonce_generation_instant: None,
        nonce: None,
        current_token: None,
    }]
}

#[derive(Debug, Clone)]
pub struct UserManager {
    admins: Arc<RwLock<[AdminUser; 1]>>,
}

impl UserManager {
    pub fn get() -> Self {
        USER_MANAGER.get_or_init(|| UserManager::default()).clone()
    }

    pub fn generate_nonce<S: AsRef<str>>(&self, username: S) -> Option<String> {
        let mut admins = self.admins.write().unwrap();
        for admin in admins.iter_mut() {
            if admin.username == username.as_ref() {
                let nonce = generate_nonce();
                admin.nonce = Some(nonce.clone());
                admin.nonce_generation_instant = Some(Instant::now());
                return Some(nonce);
            }
        }
        None
    }

    pub fn generate_token<S: AsRef<str>>(
        &mut self,
        username: S,
        password_with_nonce_hmac: S,
        ip: S,
    ) -> Option<AuthToken> {
        let mut admins = self.admins.write().unwrap();
        for admin in admins.iter_mut() {
            if admin.username != username.as_ref() {
                continue;
            }

            // Check if nonce exists and hasn't expired
            let nonce = match &admin.nonce {
                Some(n) => n.clone(),
                None => return None,
            };
            if let Some(gen_instant) = admin.nonce_generation_instant {
                if gen_instant.elapsed() > std::time::Duration::from_secs(300) {
                    // Nonce expired
                    admin.nonce = None;
                    admin.nonce_generation_instant = None;
                    return None;
                }
            } else {
                return None;
            }

            let mut mac = HmacSha256::new_from_slice(admin.password.as_bytes()).unwrap();
            mac.update(nonce.as_bytes());
            let expected = mac.finalize().into_bytes();
            let expected_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(expected);

            if expected_b64 != password_with_nonce_hmac.as_ref() {
                return None;
            }

            let token = AuthToken::new(ip.as_ref().to_string(), 300);
            admin.current_token = Some(token.clone());

            admin.nonce = None;
            admin.nonce_generation_instant = None;

            return Some(token);
        }

        None
    }
}

impl Default for UserManager {
    fn default() -> Self {
        Self {
            admins: Arc::new(RwLock::new(load_admins_users())),
        }
    }
}

pub fn check_admin_auth_from_request(
    req: &mut esp_idf_svc::http::server::Request<&mut esp_idf_svc::http::server::EspHttpConnection>,
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

    let user_manager = UserManager::get();
    let admins = user_manager.admins.read().unwrap();
    let matching_user = admins.iter().find(|admin| {
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

fn generate_nonce() -> String {
    let mut buf = [0u8; 16]; // 128-bit nonce

    // Fill 16 bytes using esp_random (returns u32)
    for chunk in buf.chunks_mut(4) {
        let r = unsafe { esp_random() }; // u32
        chunk.copy_from_slice(&r.to_le_bytes()); // convert u32 -> 4 bytes
    }

    general_purpose::URL_SAFE_NO_PAD.encode(buf) // base64 URL-safe
}
