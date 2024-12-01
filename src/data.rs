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
}

#[derive(Debug, Deserialize)]
pub struct OrderBookSummary {
    pub market: String,
    pub asset_id: String,
    pub timestamp: u64,
    pub bids: Vec<OrderSummary>,
    pub asks: Vec<OrderSummary>,
}

#[derive(Debug)]
pub struct MarketOrderArgs {
    pub token_id: String,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct OrderSummary {
    pub price: Decimal,
    pub size: Decimal,
}

impl OrderArgs {
    pub fn new(token_id: &str, price: Decimal, size: Decimal, side: Side) -> Self {
        OrderArgs {
            token_id: token_id.to_owned(),
            price,
            size,
            side,
        }
    }
}

#[derive(Debug)]
pub struct ExtraOrderArgs {
    pub fee_rate_bps: u32,
    pub nonce: U256,
    pub taker: String,
}

impl Default for ExtraOrderArgs {
    fn default() -> Self {
        ExtraOrderArgs {
            fee_rate_bps: 0,
            nonce: U256::ZERO,
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

#[derive(Debug, Deserialize, Serialize)]
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
