use crate::Decimal;
use alloy_primitives::U256;
use serde::{Deserialize, Serialize};

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

#[derive(Debug)]
pub struct OrderArgs {
    pub token_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
    pub extras: Option<OrderExtras>,
}

#[derive(Debug)]
pub struct OrderExtras {
    pub fee_rate_bps: u32,
    pub nonce: U256,
    pub expiration: u64,
    pub taker: String,
}

impl Default for OrderExtras {
    fn default() -> OrderExtras {
        OrderExtras {
            fee_rate_bps: 0,
            nonce: U256::ZERO,
            expiration: 0,
            taker: ZERO_ADDRESS.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CreateOrderOptions {
    pub tick_size: Option<Decimal>,
    pub neg_risk: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeysResponse {
    #[serde(rename = "apiKeys")]
    pub api_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MidpointResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub mid: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct SpreadResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub spread: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct TickSizeResponse {
    pub minimum_tick_size: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct NegRiskResponse {
    pub neg_risk: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Side {
    BUY = 0,
    SELL = 1,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::BUY => "BUY",
            Side::SELL => "SELL",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookParams {
    pub token_id: String,
    pub side: Side,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ApiCreds {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}
