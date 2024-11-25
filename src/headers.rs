use crate::eth_utils::{sign_clob_auth_message, EthSigner};
use crate::utils::get_current_unix_time_secs;
use crate::ClientResult;
use alloy_primitives::hex::encode_prefixed;
use alloy_primitives::U256;
use std::collections::HashMap;

const POLY_ADDR_HEADER: &str = "POLY_ADDRESS";
const POLY_SIG_HEADER: &str = "POLY_SIGNATURE";
const POLY_TS_HEADER: &str = "POLY_TIMESTAMP";
const POLY_NONCE_HEADER: &str = "POLY_NONCE";

type Headers = HashMap<String, String>;

pub fn create_l1_headers(signer: &impl EthSigner, nonce: Option<U256>) -> ClientResult<Headers> {
    let timestamp = get_current_unix_time_secs().to_string();

    let nonce = nonce.unwrap_or(U256::from_be_slice(&[0]));
    let signature = sign_clob_auth_message(signer, timestamp.clone(), nonce)?;
    let address = encode_prefixed(signer.address().as_slice());

    Ok(HashMap::from([
        (POLY_ADDR_HEADER.to_owned(), address),
        (POLY_SIG_HEADER.to_owned(), signature),
        (POLY_TS_HEADER.to_owned(), timestamp),
        (POLY_NONCE_HEADER.to_owned(), nonce.to_string()),
    ]))
}
