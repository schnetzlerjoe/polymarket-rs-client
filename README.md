# Polymarket Rust Client &emsp; [![Latest Version]][crates.io] [![Docs Badge]][docs]

[Latest Version]: https://img.shields.io/crates/v/polymarket-rs-client.svg

[crates.io]: https://crates.io/crates/polymarket-rs-client

[Docs Badge]: https://docs.rs/polymarket-rs-client/badge.svg

[docs]: https://docs.rs/rust_decimal
An async rust client for interacting with Polymarket.

## Installing
```sh
cargo add polymarket-rs-client
```
The client internally uses a reqwest [`Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html), so you will also need the `tokio` runtime.
```sh
cargo add -F rt-multi-thread,macros tokio
```

## Usage
Create an instance of the `ClobClient` to interact with the [CLOB API](https://docs.polymarket.com/#clob-api). Note that the prerequisite allowances must be set before creating and sending an order as described [here](https://github.com/Polymarket/py-clob-client?tab=readme-ov-file#allowances).


```rust
use polymarket_rs_client::ClobClient;

#[tokio::main]
async fn main() {
    let client = ClobClient::new("https://clob.polymarket.com");

    let o = client.get_sampling_markets(None).await;
    dbg!(o);
}
```

The `ClobClient` implements the same API as the [official python client](https://github.com/Polymarket/py-clob-client). All available functions are listed in the [docs](https://docs.rs/polymarket-rs-client/latest/polymarket_rs_client/struct.ClobClient.html).

## Benchmarks

TODO
