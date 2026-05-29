use std::sync::{Arc, RwLock};

use base64::engine::general_purpose;
use base64::Engine;
use sha2::Digest;

pub const AUTH_TOKEN_TTL_SECS: u64 = 120;
pub const NONCE_TTL_SECS: u64 = 60;

/// SHA-512(password+nonce) base64url for password `"1234"` and nonce `"test-nonce-fixed"`.
/// Shared with frontend Vitest for cross-language parity.
pub const PASSWORD_NONCE_HASH_VECTOR: &str =
    "o7cndA46XRWj_WVFQ41iFdK_JHdjBD7hNjvk023on4mUwEQLa_GU-hbJQUv8lW9xLIAMmTiOukTLMudtWoMtYQ";

pub type RngFn = fn(&mut [u8]);
pub type NowFn = fn() -> u64;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthToken {
    hash: String,
    expires_at: u64,
    ip: String,
}

impl AuthToken {
    pub fn new(ip: impl AsRef<str>, ttl_secs: u64, now_secs: u64, rng: RngFn) -> Self {
        let mut buf = [0u8; 16];
        rng(&mut buf);
        let hash = general_purpose::URL_SAFE_NO_PAD.encode(buf);

        Self {
            hash,
            ip: ip.as_ref().to_string(),
            expires_at: now_secs + ttl_secs,
        }
    }

    pub fn into_string(self) -> String {
        let payload = format!("{}|{}|{}", self.hash, self.expires_at, self.ip);
        general_purpose::URL_SAFE_NO_PAD.encode(payload)
    }

    pub fn from_string(token: &str, now_secs: u64) -> Option<Self> {
        let payload_bytes = general_purpose::URL_SAFE_NO_PAD.decode(token).ok()?;
        let payload = String::from_utf8(payload_bytes).ok()?;

        let mut payload_parts = payload.split('|');
        let hash = payload_parts.next()?.to_string();
        let expires_at = payload_parts.next()?.parse::<u64>().ok()?;
        let ip = payload_parts.next()?.to_string();

        if payload_parts.next().is_some() {
            return None;
        }

        if now_secs > expires_at {
            return None;
        }

        Some(AuthToken {
            hash,
            expires_at,
            ip,
        })
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }
}

impl From<AuthToken> for String {
    fn from(value: AuthToken) -> Self {
        value.into_string()
    }
}

#[derive(Debug, Clone)]
struct AdminUser {
    username: String,
    password: String,
    nonce: Option<String>,
    nonce_issued_at: Option<u64>,
    current_token: Option<AuthToken>,
}

#[derive(Debug, Clone)]
pub struct UserManager {
    admins: Arc<RwLock<Vec<AdminUser>>>,
    rng: RngFn,
    now: NowFn,
}

impl UserManager {
    pub fn new(users: Vec<(String, String)>, rng: RngFn, now: NowFn) -> Self {
        let admins = users
            .into_iter()
            .map(|(username, password)| AdminUser {
                username,
                password,
                nonce: None,
                nonce_issued_at: None,
                current_token: None,
            })
            .collect();

        Self {
            admins: Arc::new(RwLock::new(admins)),
            rng,
            now,
        }
    }

    pub fn hash_password_with_nonce(password: &str, nonce: &str) -> String {
        let digest = sha2::Sha512::digest(format!("{}+{}", password, nonce).as_bytes());
        general_purpose::URL_SAFE_NO_PAD.encode(digest)
    }

    pub fn generate_nonce(&self, username: impl AsRef<str>) -> Option<String> {
        let mut admins = self.admins.write().ok()?;
        let admin = admins
            .iter_mut()
            .find(|a| a.username == username.as_ref())?;

        let mut buf = [0u8; 16];
        (self.rng)(&mut buf);
        let nonce = general_purpose::URL_SAFE_NO_PAD.encode(buf);

        admin.nonce = Some(nonce.clone());
        admin.nonce_issued_at = Some((self.now)());
        Some(nonce)
    }

    pub fn generate_token(
        &self,
        username: impl AsRef<str>,
        password_with_nonce_hashed: impl AsRef<str>,
        ip: impl AsRef<str>,
    ) -> anyhow::Result<AuthToken> {
        let mut admins = self
            .admins
            .write()
            .map_err(|err| anyhow::anyhow!("Lock poisoned: {}", err))?;

        let target = admins
            .iter_mut()
            .find(|a| a.username == username.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Credenciais invalidas"))?;

        let now = (self.now)();

        if let Some(issued_at) = target.nonce_issued_at {
            if now.saturating_sub(issued_at) > NONCE_TTL_SECS {
                target.nonce = None;
                target.nonce_issued_at = None;
                return Err(anyhow::anyhow!("Credenciais invalidas"));
            }
        }

        let nonce = target
            .nonce
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Credenciais invalidas"))?;

        let correct_hash = Self::hash_password_with_nonce(&target.password, nonce);

        if password_with_nonce_hashed.as_ref() != correct_hash {
            target.nonce = None;
            target.nonce_issued_at = None;
            return Err(anyhow::anyhow!("Credenciais invalidas"));
        }

        target.nonce = None;
        target.nonce_issued_at = None;

        let token = AuthToken::new(ip, AUTH_TOKEN_TTL_SECS, now, self.rng);
        target.current_token = Some(token.clone());
        Ok(token)
    }

    pub fn validate_token(&self, token: &AuthToken, client_ip: &str) -> bool {
        if token.ip() != client_ip {
            return false;
        }

        let now = (self.now)();
        if now > token.expires_at() {
            return false;
        }

        let admins = match self.admins.read() {
            Ok(a) => a,
            Err(_) => return false,
        };

        admins.iter().any(|admin| {
            admin
                .current_token
                .as_ref()
                .map_or(false, |t| t == token)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    thread_local! {
        static TEST_NOW: Cell<u64> = const { Cell::new(1_700_000_000) };
    }

    static mut RNG_COUNTER: u8 = 0;

    fn test_rng(buf: &mut [u8]) {
        unsafe {
            for b in buf.iter_mut() {
                *b = RNG_COUNTER;
                RNG_COUNTER = RNG_COUNTER.wrapping_add(1);
            }
        }
    }

    fn fixed_now() -> u64 {
        TEST_NOW.with(|t| t.get())
    }

    fn set_test_now(secs: u64) {
        TEST_NOW.with(|t| t.set(secs));
    }

    fn manager_with_user() -> UserManager {
        set_test_now(1_700_000_000);
        UserManager::new(
            vec![("root".to_string(), "1234".to_string())],
            test_rng,
            fixed_now,
        )
    }

    #[test]
    fn password_nonce_hash_vector() {
        let hash = UserManager::hash_password_with_nonce("1234", "test-nonce-fixed");
        assert_eq!(hash, PASSWORD_NONCE_HASH_VECTOR);
    }

    #[test]
    fn token_roundtrip() {
        let token = AuthToken::new("192.168.1.10", AUTH_TOKEN_TTL_SECS, fixed_now(), test_rng);
        let encoded = token.clone().into_string();
        let parsed = AuthToken::from_string(&encoded, fixed_now()).unwrap();
        assert_eq!(parsed, token);
    }

    #[test]
    fn token_expired() {
        let token = AuthToken::new("10.0.0.1", 10, fixed_now(), test_rng);
        let encoded = token.into_string();
        assert!(AuthToken::from_string(&encoded, fixed_now() + 11).is_none());
    }

    #[test]
    fn login_happy_path() {
        let mgr = manager_with_user();
        let nonce = mgr.generate_nonce("root").unwrap();
        let hash = UserManager::hash_password_with_nonce("1234", &nonce);
        let token = mgr
            .generate_token("root", hash, "192.168.1.5")
            .unwrap();
        assert!(mgr.validate_token(&token, "192.168.1.5"));
    }

    #[test]
    fn login_wrong_hash() {
        let mgr = manager_with_user();
        mgr.generate_nonce("root").unwrap();
        assert!(mgr
            .generate_token("root", "wrong-hash", "192.168.1.5")
            .is_err());
        let nonce = mgr.generate_nonce("root").unwrap();
        let hash = UserManager::hash_password_with_nonce("1234", &nonce);
        let token = mgr
            .generate_token("root", hash, "192.168.1.5")
            .unwrap();
        assert!(mgr.validate_token(&token, "192.168.1.5"));
    }

    #[test]
    fn login_nonce_expired() {
        let mgr = manager_with_user();
        let nonce = mgr.generate_nonce("root").unwrap();
        let hash = UserManager::hash_password_with_nonce("1234", &nonce);
        set_test_now(1_700_000_000 + NONCE_TTL_SECS + 1);
        assert!(mgr.generate_token("root", hash, "192.168.1.5").is_err());
    }

    #[test]
    fn login_nonce_single_use() {
        let mgr = manager_with_user();
        let nonce = mgr.generate_nonce("root").unwrap();
        let hash = UserManager::hash_password_with_nonce("1234", &nonce);
        assert!(mgr.generate_token("root", hash.clone(), "192.168.1.5").is_ok());
        assert!(mgr.generate_token("root", hash, "192.168.1.5").is_err());
    }

    #[test]
    fn validate_wrong_ip() {
        let mgr = manager_with_user();
        let nonce = mgr.generate_nonce("root").unwrap();
        let hash = UserManager::hash_password_with_nonce("1234", &nonce);
        let token = mgr
            .generate_token("root", hash, "10.0.0.1")
            .unwrap();
        assert!(!mgr.validate_token(&token, "10.0.0.2"));
    }

    #[test]
    fn validate_stale_token() {
        let mgr = manager_with_user();
        let nonce = mgr.generate_nonce("root").unwrap();
        let hash = UserManager::hash_password_with_nonce("1234", &nonce);
        let old = mgr
            .generate_token("root", hash.clone(), "10.0.0.1")
            .unwrap();

        let nonce2 = mgr.generate_nonce("root").unwrap();
        let hash2 = UserManager::hash_password_with_nonce("1234", &nonce2);
        let new = mgr
            .generate_token("root", hash2, "10.0.0.1")
            .unwrap();

        assert!(!mgr.validate_token(&old, "10.0.0.1"));
        assert!(mgr.validate_token(&new, "10.0.0.1"));
    }
}
