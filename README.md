# Polymarket Rust Client &emsp; [![Latest Version]][crates.io] [![Docs Badge]][docs]

[Latest Version]: https://img.shields.io/crates/v/polymarket-rs-client.svg
[crates.io]: https://crates.io/crates/polymarket-rs-client
[Docs Badge]: https://docs.rs/polymarket-rs-client/badge.svg
[docs]: https://docs.rs/polymarket-rs-client

An async rust client for interacting with [Polymarket](https://polymarket.com/).

> [!NOTE]
> This code is still in active developement and alpha quality. Use at your own risk.

## Why use this instead of the official client?

1. You get to write Rust!
2. Most calls are anywhere from 1.5x to upto 4x faster.
3. Upto 10x less memory usage.

Some benchmarks on my machine:
| | polymarket-rs-client | Official Python client |
|-------------------------------------------|-------------------------------------------------------------|------------------------------------------------------------|
| Create a order with EIP-712 signature. | **266.5 ms ± 28.6 ms** | 1.127 s ± 0.047 s |
| Fetch and parse json(simplified markets). | **404.5 ms ± 22.9 ms** | 1.366 s ± 0.048 s |
| Fetch markets. Mem usage | **88,053 allocs, 81,823 frees, 15,945,966 bytes allocated** | 211,898 allocs, 202,962 frees, 128,457,588 bytes allocated |

## Installing

```sh
cargo add polymarket-rs-client
```

The client internally uses a reqwest [`Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html), so you will also need the `tokio` runtime.

```sh
cargo add -F rt-multi-thread,macros tokio

```

For representing order amounts and sizes, the client uses the `rust-decimal` crate. It is recommended to install this crate as well.

```sh
cargo add rust-decimal
```

## Usage

Create an instance of the `ClobClient` to interact with the [CLOB API](https://docs.polymarket.com/#clob-api). Note that the prerequisite allowances must be set before creating and sending an order as described [here](https://github.com/Polymarket/py-clob-client?tab=readme-ov-file#allowances).

```rust
use polymarket_rs_client::ClobClient;

use std::env;

const HOST: &str = "https://clob.polymarket.com";
const POLYGON: u64 = 137;

#[tokio::main]
async fn main() {
    let private_key = env::var("PK").unwrap();
    let nonce = None;

    let mut client = ClobClient::with_l1_headers(HOST, &private_key, POLYGON);
    let keys = client.create_or_derive_api_key(nonce).await.unwrap();
    client.set_api_creds(keys);

    let o = client.get_sampling_markets(None).await.unwrap();
    dbg!(o);
}
```

The `ClobClient` implements the same API as the [official python client](https://github.com/Polymarket/py-clob-client). All available functions are listed in the [docs](https://docs.rs/polymarket-rs-client/latest/polymarket_rs_client/struct.ClobClient.html).
