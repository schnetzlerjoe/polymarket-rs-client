use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE, Engine};
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn get_current_unix_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn build_hmac_signature(
    secret: &str,
    timestamp: u64,
    method: &str,
    req_path: &str,
    body: Option<&str>,
) -> Result<String> {
    let decoded = URL_SAFE
        .decode(secret)
        .context("Can't decode secret to base64")?;
    // TODO: test body str
    let message = match body {
        None => format!("{timestamp}{method}{req_path}"),
        Some(s) => {
            let s = s.replace("'", r#"""#);
            format!("{timestamp}{method}{req_path}{s}")
        }
    };

    let mut mac = HmacSha256::new_from_slice(&decoded).context("HMAC init error")?;
    mac.update(message.as_bytes());

    let result = mac.finalize();

    Ok(URL_SAFE.encode(&result.into_bytes()[..]))
}
