use alloy_primitives::hex::encode_prefixed;
pub use alloy_primitives::U256;
use alloy_signer_local::PrivateKeySigner;
pub use anyhow::{anyhow, Context, Result as ClientResult};
use config::get_contract_config;
use orders::OrderBuilder;
use orders::SignedOrderRequest;
use reqwest::header::HeaderName;
use reqwest::Client;
use reqwest::Method;
use reqwest::RequestBuilder;
pub use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

mod config;
mod data;
mod eth_utils;
mod headers;
mod orders;
mod utils;

pub use data::*;
pub use eth_utils::EthSigner;
use headers::{create_l1_headers, create_l2_headers};

#[derive(Default)]
pub struct ClobClient {
    host: String,
    http_client: Client,
    signer: Option<Box<dyn EthSigner>>,
    chain_id: Option<u64>,
    api_creds: Option<ApiCreds>,
    order_builder: Option<OrderBuilder>,
}

impl ClobClient {
    // TODO: initial headers, gzip
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
            http_client: Client::new(),
            ..Default::default()
        }
    }
    pub fn with_l1_headers(host: &str, key: &str, chain_id: u64) -> Self {
        let signer = Box::new(
            key.parse::<PrivateKeySigner>()
                .expect("Invalid private key"),
        );
        Self {
            host: host.to_owned(),
            http_client: Client::new(),
            signer: Some(signer.clone()),
            chain_id: Some(chain_id),
            api_creds: None,
            order_builder: Some(OrderBuilder::new(signer, None, None)),
        }
    }

    pub fn with_l2_headers(host: &str, key: &str, chain_id: u64, api_creds: ApiCreds) -> Self {
        let signer = Box::new(
            key.parse::<PrivateKeySigner>()
                .expect("Invalid private key"),
        );
        Self {
            host: host.to_owned(),
            http_client: Client::new(),
            signer: Some(signer.clone()),
            chain_id: Some(chain_id),
            api_creds: Some(api_creds),
            order_builder: Some(OrderBuilder::new(signer, None, None)),
        }
    }
    pub fn set_api_creds(&mut self, api_creds: ApiCreds) {
        self.api_creds = Some(api_creds);
    }

    #[inline]
    fn get_l1_parameters(&self) -> (&impl EthSigner, u64) {
        let signer = self.signer.as_ref().expect("Signer is not set");
        let chain_id = self.chain_id.expect("Chain id is not set");
        (signer, chain_id)
    }

    #[inline]
    fn get_l2_parameters(&self) -> (&impl EthSigner, &ApiCreds) {
        let signer = self.signer.as_ref().expect("Signer is not set");
        (
            signer,
            self.api_creds.as_ref().expect("API credentials not set."),
        )
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

    fn create_request_with_headers(
        &self,
        method: Method,
        endpoint: &str,
        headers: impl Iterator<Item = (&'static str, String)>,
    ) -> RequestBuilder {
        let req = self
            .http_client
            .request(method, format!("{}{endpoint}", &self.host));

        headers.fold(req, |r, (k, v)| r.header(HeaderName::from_static(k), v))
    }

    pub async fn get_ok(&self) -> bool {
        self.http_client
            .get(format!("{}/", &self.host))
            .send()
            .await
            .is_ok()
    }

    pub async fn get_server_time(&self) -> ClientResult<u64> {
        let resp = self
            .http_client
            .get(format!("{}/time", &self.host))
            .send()
            .await?
            .text()
            .await?
            .parse::<u64>()?;
        Ok(resp)
    }

    pub async fn create_api_key(&self, nonce: Option<U256>) -> ClientResult<ApiCreds> {
        let method = Method::POST;
        let endpoint = "/auth/api-key";
        let (signer, _) = self.get_l1_parameters();
        let headers = create_l1_headers(signer, nonce)?;

        let req = self.create_request_with_headers(method, endpoint, headers.into_iter());

        Ok(req.send().await?.json::<ApiCreds>().await?)
    }

    pub async fn derive_api_key(&self, nonce: Option<U256>) -> ClientResult<ApiCreds> {
        let method = Method::GET;
        let endpoint = "/auth/derive-api-key";
        let (signer, _) = self.get_l1_parameters();
        let headers = create_l1_headers(signer, nonce)?;

        let req = self.create_request_with_headers(method, endpoint, headers.into_iter());

        Ok(req.send().await?.json::<ApiCreds>().await?)
    }

    pub async fn create_or_derive_api_key(&self, nonce: Option<U256>) -> ClientResult<ApiCreds> {
        let creds = self.create_api_key(nonce).await;
        if creds.is_err() {
            return self.derive_api_key(nonce).await;
        }
        creds
    }

    pub async fn get_api_keys(&self) -> ClientResult<Vec<String>> {
        let method = Method::GET;
        let endpoint = "/auth/api-keys";
        let (signer, creds) = self.get_l2_parameters();
        let headers = create_l2_headers::<Value>(signer, creds, method.as_str(), endpoint, None)?;

        let req = self.create_request_with_headers(method, endpoint, headers.into_iter());

        Ok(req.send().await?.json::<ApiKeysResponse>().await?.api_keys)
    }

    pub async fn delete_api_key(&self) -> ClientResult<String> {
        let method = Method::DELETE;
        let endpoint = "/auth/api-key";
        let (signer, creds) = self.get_l2_parameters();
        let headers = create_l2_headers::<Value>(signer, creds, method.as_str(), endpoint, None)?;
        let req = self.create_request_with_headers(method, endpoint, headers.into_iter());

        Ok(req.send().await?.text().await?)
    }

    pub async fn get_midpoint(&self, token_id: &str) -> ClientResult<MidpointResponse> {
        Ok(self
            .http_client
            .get(format!("{}/midpoint", &self.host))
            .query(&[("token_id", token_id)])
            .send()
            .await?
            .json::<MidpointResponse>()
            .await?)
    }

    pub async fn get_midpoints(
        &self,
        token_ids: &[String],
    ) -> ClientResult<HashMap<String, Decimal>> {
        let v = token_ids
            .iter()
            .map(|b| HashMap::from([("token_id", b.clone())]))
            .collect::<Vec<HashMap<&str, String>>>();

        Ok(self
            .http_client
            .post(format!("{}/midpoints", &self.host))
            .json(&v)
            .send()
            .await?
            .json::<HashMap<String, Decimal>>()
            .await?)
    }

    pub async fn get_price(&self, token_id: &str, side: Side) -> ClientResult<PriceResponse> {
        Ok(self
            .http_client
            .get(format!("{}/price", &self.host))
            .query(&[("token_id", token_id)])
            .query(&[("side", side.as_str())])
            .send()
            .await?
            .json::<PriceResponse>()
            .await?)
    }
    pub async fn get_prices(
        &self,
        book_params: &[BookParams],
    ) -> ClientResult<HashMap<String, HashMap<Side, Decimal>>> {
        let v = book_params
            .iter()
            .map(|b| {
                HashMap::from([
                    ("token_id", b.token_id.clone()),
                    ("side", b.side.as_str().to_owned()),
                ])
            })
            .collect::<Vec<HashMap<&str, String>>>();

        Ok(self
            .http_client
            .post(format!("{}/prices", &self.host))
            .json(&v)
            .send()
            .await?
            .json::<HashMap<String, HashMap<Side, Decimal>>>()
            .await?)
    }

    pub async fn get_spread(&self, token_id: &str) -> ClientResult<SpreadResponse> {
        Ok(self
            .http_client
            .get(format!("{}/spread", &self.host))
            .query(&[("token_id", token_id)])
            .send()
            .await?
            .json::<SpreadResponse>()
            .await?)
    }

    pub async fn get_spreads(
        &self,
        token_ids: &[String],
    ) -> ClientResult<HashMap<String, Decimal>> {
        let v = token_ids
            .iter()
            .map(|b| HashMap::from([("token_id", b.clone())]))
            .collect::<Vec<HashMap<&str, String>>>();

        Ok(self
            .http_client
            .post(format!("{}/spreads", &self.host))
            .json(&v)
            .send()
            .await?
            .json::<HashMap<String, Decimal>>()
            .await?)
    }

    // cache
    pub async fn get_tick_size(&self, token_id: &str) -> ClientResult<Decimal> {
        Ok(self
            .http_client
            .get(format!("{}/tick-size", &self.host))
            .query(&[("token_id", token_id)])
            .send()
            .await?
            .json::<TickSizeResponse>()
            .await?
            .minimum_tick_size)
    }
    // Cache
    pub async fn get_neg_risk(&self, token_id: &str) -> ClientResult<bool> {
        Ok(self
            .http_client
            .get(format!("{}/neg-risk", &self.host))
            .query(&[("token_id", token_id)])
            .send()
            .await?
            .json::<NegRiskResponse>()
            .await?
            .neg_risk)
    }

    async fn resolve_tick_size(
        &self,
        token_id: &str,
        tick_size: Option<Decimal>,
    ) -> ClientResult<Decimal> {
        let min_tick_size = self
            .get_tick_size(token_id)
            .await
            .context("Error fetching tick size")?;

        match tick_size {
            None => Ok(min_tick_size),
            Some(t) => {
                if t < min_tick_size {
                    Err(anyhow!("Tick size {t} is smaller than min_tick_size {min_tick_size} for token_id: {token_id}"))
                } else {
                    Ok(t)
                }
            }
        }
    }

    async fn get_filled_order_options(
        &self,
        token_id: &str,
        options: Option<&CreateOrderOptions>,
    ) -> ClientResult<CreateOrderOptions> {
        let (tick_size, neg_risk) = match options {
            Some(o) => (o.tick_size, o.neg_risk),
            None => (None, None),
        };

        let tick_size = self.resolve_tick_size(token_id, tick_size).await?;

        let neg_risk = match neg_risk {
            Some(nr) => nr,
            None => self.get_neg_risk(token_id).await?,
        };

        Ok(CreateOrderOptions {
            neg_risk: Some(neg_risk),
            tick_size: Some(tick_size),
        })
    }

    fn is_price_in_range(&self, price: Decimal, tick_size: Decimal) -> bool {
        let min_price = tick_size;
        let max_price = Decimal::ONE - tick_size;

        if price < min_price || price > max_price {
            return false;
        }
        true
    }

    pub async fn create_order(
        &self,
        order_args: &OrderArgs,
        expiration: Option<u64>,
        extras: Option<ExtraOrderArgs>,
        options: Option<&CreateOrderOptions>,
    ) -> ClientResult<SignedOrderRequest> {
        let (_, chain_id) = self.get_l1_parameters();

        let create_order_options = self
            .get_filled_order_options(order_args.token_id.as_ref(), options)
            .await?;
        let expiration = expiration.unwrap_or(0);
        let extras = extras.unwrap_or_default();

        if !self.is_price_in_range(
            order_args.price,
            create_order_options.tick_size.expect("Should be filled"),
        ) {
            return Err(anyhow!("Price is not in range of tick_size"));
        }

        self.order_builder
            .as_ref()
            .expect("OrderBuilder not set")
            .create_order(
                chain_id,
                order_args,
                expiration,
                &extras,
                create_order_options,
            )
    }

    pub async fn get_order_book(&self, token_id: &str) -> ClientResult<OrderBookSummary> {
        Ok(self
            .http_client
            .get(format!("{}/tick-size", &self.host))
            .query(&[("token_id", token_id)])
            .send()
            .await?
            .json::<OrderBookSummary>()
            .await?)
    }

    async fn calculate_market_price(
        &self,
        token_id: &str,
        side: Side,
        amount: Decimal,
    ) -> ClientResult<Decimal> {
        let book = self.get_order_book(token_id).await?;
        let ob = self
            .order_builder
            .as_ref()
            .expect("No orderBuilder set for client!");
        match side {
            Side::BUY => ob.calculate_market_price(&book.asks, amount),
            Side::SELL => ob.calculate_market_price(&book.bids, amount),
        }
    }

    pub async fn create_market_order(
        &self,
        order_args: &MarketOrderArgs,
        extras: Option<ExtraOrderArgs>,
        options: Option<&CreateOrderOptions>,
    ) -> ClientResult<SignedOrderRequest> {
        let (_, chain_id) = self.get_l1_parameters();

        let create_order_options = self
            .get_filled_order_options(order_args.token_id.as_ref(), options)
            .await?;

        let extras = extras.unwrap_or_default();
        let price = self
            .calculate_market_price(&order_args.token_id, Side::BUY, order_args.amount)
            .await?;
        if !self.is_price_in_range(
            price,
            create_order_options.tick_size.expect("Should be filled"),
        ) {
            return Err(anyhow!("Price is not in range of tick_size"));
        }

        self.order_builder
            .as_ref()
            .expect("OrderBuilder not set")
            .create_market_order(chain_id, order_args, price, &extras, create_order_options)
    }

    pub async fn post_order(
        &self,
        order: SignedOrderRequest,
        order_type: OrderType,
    ) -> ClientResult<Value> {
        let (signer, creds) = self.get_l2_parameters();
        let body = PostOrder::new(order, creds.api_key.clone(), order_type);

        let method = Method::POST;
        let endpoint = "/order";

        let headers = create_l2_headers(signer, creds, method.as_str(), endpoint, Some(&body))?;

        let req = self.create_request_with_headers(method, endpoint, headers.into_iter());

        Ok(req.json(&body).send().await?.json::<Value>().await?)
    }

    // create_and_post_order
    //
    // return type
    pub async fn cancel(&self, order_id: &str) -> ClientResult<bool> {
        let (signer, creds) = self.get_l2_parameters();
        let body = HashMap::from([("orderID", order_id)]);

        let method = Method::DELETE;
        let endpoint = "/orders";

        let headers = create_l2_headers(signer, creds, method.as_str(), endpoint, Some(&body))?;

        let req = self.create_request_with_headers(method, endpoint, headers.into_iter());
        req.json(&body).send().await?;

        Ok(false)
    }
}
