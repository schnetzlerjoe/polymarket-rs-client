use alloy_primitives::Address;
use alloy_primitives::U256;
use rand::thread_rng;
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy::{AwayFromZero, MidpointTowardZero, ToZero};

use crate::config::get_contract_config;
use crate::eth_utils::sign_order_message;
use crate::eth_utils::Order;
use crate::utils::get_current_unix_time_secs;
use crate::{CreateOrderOptions, EthSigner, OrderArgs, Side};

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Copy, Clone)]
pub enum SigType {
    // ECDSA EIP712 signatures signed by EOAs
    Eoa = 0,
    // EIP712 signatures signed by EOAs that own Polymarket Proxy wallets
    PolyProxy = 1,
    // EIP712 signatures signed by EOAs that own Polymarket Gnosis safes
    PolyGnosisSafe = 2,
}

pub struct OrderBuilder {
    signer: Box<dyn EthSigner>,
    sig_type: SigType,
    funder: Address,
}

pub struct RoundConfig {
    price: u32,
    size: u32,
    amount: u32,
}

static ROUNDING_CONFIG: LazyLock<HashMap<Decimal, RoundConfig>> = LazyLock::new(|| {
    HashMap::from([
        (
            Decimal::from_str("0.1").unwrap(),
            RoundConfig {
                price: 1,
                size: 2,
                amount: 3,
            },
        ),
        (
            Decimal::from_str("0.01").unwrap(),
            RoundConfig {
                price: 2,
                size: 2,
                amount: 4,
            },
        ),
        (
            Decimal::from_str("0.001").unwrap(),
            RoundConfig {
                price: 3,
                size: 2,
                amount: 5,
            },
        ),
        (
            Decimal::from_str("0.0001").unwrap(),
            RoundConfig {
                price: 4,
                size: 2,
                amount: 6,
            },
        ),
    ])
});

fn decimal_to_token_u32(amt: Decimal) -> u32 {
    let mut amt = Decimal::from_scientific("1e6").expect("1e6 is not scientific") * amt;
    if amt.scale() > 0 {
        amt = amt.round_dp_with_strategy(0, MidpointTowardZero);
    }
    amt.try_into().expect("Couldn't round decimal to integer")
}

impl OrderBuilder {
    pub fn new(
        signer: Box<dyn EthSigner>,
        sig_type: Option<SigType>,
        funder: Option<Address>,
    ) -> Self {
        let sig_type = if let Some(st) = sig_type {
            st
        } else {
            SigType::Eoa
        };

        let funder = if let Some(f) = funder {
            f
        } else {
            signer.address()
        };

        OrderBuilder {
            signer,
            sig_type,
            funder,
        }
    }

    fn fix_amount_rounding(&self, mut amt: Decimal, round_config: &RoundConfig) -> Decimal {
        if amt.scale() > round_config.amount {
            amt = amt.round_dp_with_strategy(round_config.amount + 4, AwayFromZero);
            if amt.scale() > round_config.amount {
                amt = amt.round_dp_with_strategy(round_config.amount, ToZero);
            }
        }
        amt
    }

    fn get_order_amounts(
        &self,
        side: Side,
        size: Decimal,
        price: Decimal,
        round_config: &RoundConfig,
    ) -> (u32, u32) {
        let raw_price = price.round_dp_with_strategy(round_config.price, MidpointTowardZero);

        match side {
            Side::BUY => {
                let raw_taker_amt = size.round_dp_with_strategy(round_config.size, ToZero);
                let raw_maker_amt = raw_taker_amt * raw_price;
                let raw_maker_amt = self.fix_amount_rounding(raw_maker_amt, round_config);
                (
                    decimal_to_token_u32(raw_maker_amt),
                    decimal_to_token_u32(raw_taker_amt),
                )
            }
            Side::SELL => {
                let raw_maker_amt = size.round_dp_with_strategy(round_config.size, ToZero);
                let raw_taker_amt = raw_maker_amt * raw_price;
                let raw_taker_amt = self.fix_amount_rounding(raw_taker_amt, round_config);

                (
                    decimal_to_token_u32(raw_maker_amt),
                    decimal_to_token_u32(raw_taker_amt),
                )
            }
        }
    }

    pub fn create_order(&self, chain_id: u64, order_args: OrderArgs, options: CreateOrderOptions) {
        let (maker_amount, taker_amount) = self.get_order_amounts(
            order_args.side,
            order_args.size,
            order_args.price,
            &ROUNDING_CONFIG[&options
                .tick_size
                .expect("Cannot create order without tick size")],
        );

        let contract_config = get_contract_config(
            chain_id,
            options
                .neg_risk
                .expect("Cannot create order without neg_risk"),
        )
        .expect("No contract found with given chain_id and neg_risk");
    }

    fn build_signed_order(
        &self,
        data: OrderArgs,
        chain_id: u64,
        exchange: Address,
        maker_amount: u32,
        taker_amount: u32,
    ) {
        let extras = data.extras.expect("Need OrderExtras to be populated");
        // Does address checksum matter?
        let order = Order {
            salt: generate_seed(),
            maker: self.funder,
            signer: self.signer.address(),
            taker: Address::from_str(extras.taker.as_ref()).unwrap(),
            tokenId: U256::from_str_radix(data.token_id.as_ref(), 10)
                .expect("Incorrect tokenId format"),
            makerAmount: U256::from(maker_amount),
            takerAmount: U256::from(taker_amount),
            expiration: U256::from(extras.expiration),
            nonce: extras.nonce,
            feeRateBps: U256::from(extras.fee_rate_bps),
            side: data.side as u8,
            signatureType: self.sig_type as u8,
        };

        let signature = sign_order_message(&self.signer, order, chain_id, exchange);
        // TODO: Create hashmap in SignedOrder dict function
    }
}

pub fn generate_seed() -> U256 {
    let mut rng = thread_rng();
    let y: f64 = rng.gen();
    let a: f64 = get_current_unix_time_secs() as f64 * y;
    U256::from(a)
}
