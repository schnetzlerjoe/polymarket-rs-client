use alloy_primitives::hex::encode_prefixed;
use alloy_signer_local::PrivateKeySigner;
pub use anyhow::{Context, Result as ClientResult};
use config::get_contract_config;
use serde::{Deserialize, Serialize};
use ureq::{get, post};

mod config;
mod eth_utils;
mod headers;
mod utils;

use headers::create_l1_headers;

pub use eth_utils::EthSigner;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ApiCreds {
    #[serde(rename = "apiKey")]
    api_key: String,
    secret: String,
    passphrase: String,
}

#[derive(Default)]
pub struct ClobClient {
    host: String,
    signer: Option<Box<dyn EthSigner>>,
    chain_id: Option<u64>,
}

impl ClobClient {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
            ..Default::default()
        }
    }
    pub fn with_l1_headers(host: &str, key: &str, chain_id: u64) -> Self {
        Self {
            host: host.to_owned(),
            signer: Some(Box::new(
                key.parse::<PrivateKeySigner>()
                    .expect("Invalid private key"),
            )),
            chain_id: Some(chain_id),
        }
    }

    fn get_l1_parameters(&self) -> (&impl EthSigner, u64) {
        let signer = self.signer.as_ref().expect("Signer is not set");
        let chain_id = self.chain_id.expect("Chain id is not set");
        (signer, chain_id)
    }

    pub fn get_address(&self) -> Option<String> {
        Some(encode_prefixed(self.signer.as_ref()?.address().as_slice()))
    }

    pub fn get_collateral_address(&self) -> Option<String> {
        Some(get_contract_config(self.chain_id?, false)?.collateral)
    }

    pub fn get_conditional_address(&self) -> Option<String> {
        Some(get_contract_config(self.chain_id?, false)?.conditional_tokens)
    }

    pub fn get_exchange_address(&self) -> Option<String> {
        Some(get_contract_config(self.chain_id?, false)?.exchange)
    }

    pub fn get_ok(&self) -> bool {
        get(&format!("{}/", &self.host)).call().is_ok()
    }

    pub fn get_server_time(&self) -> ClientResult<u64> {
        let resp = get(&format!("{}/time", &self.host))
            .call()?
            .into_string()?
            .parse::<u64>()?;
        Ok(resp)
    }

    pub fn create_api_key(&self) -> ClientResult<ApiCreds> {
        let (signer, _) = self.get_l1_parameters();

        let req = post(&format!("{}/auth/api-key", &self.host));
        let headers = create_l1_headers(signer, None)?;

        let req = headers.iter().fold(req, |r, (k, v)| r.set(k, v));

        Ok(req.call()?.into_json::<ApiCreds>()?)
    }

    pub fn derive_api_key(&self) -> ClientResult<ApiCreds> {
        let (signer, _) = self.get_l1_parameters();

        let req = get(&format!("{}/auth/derive-api-key", &self.host));
        let headers = create_l1_headers(signer, None)?;

        let req = headers.iter().fold(req, |r, (k, v)| r.set(k, v));

        Ok(req.call()?.into_json::<ApiCreds>()?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::get_current_unix_time_secs;

    #[test]
    fn server_time_matches_local() {
        let host = "https://clob.polymarket.com";
        //let polygon = 137;
        let client = ClobClient::new(host);
        let curr_time = get_current_unix_time_secs();
        assert!((client.get_server_time().unwrap() as i64 - curr_time as i64).abs() < 2);
    }
}
