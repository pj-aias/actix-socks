# Actix SOCKS

SOCKSv5 support for actix-client.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
actix-socks = "0.1.0"
```

### Example: using a TOR proxy to connect to a hidden service

```rust
let client = actix_web::client::ClientBuilder::new()
    .connector(
        actix_web::client::Connector::new()
            .connector(actix_socks::SocksConnector("127.0.0.1:9050"))
            .timeout(std::time::Duration::from_secs(60))
            .finish(),
    )
    .finish();
let res = client
    .get("http://facebookcorewwwi.onion")
    .send()
    .await
    .unwrap();
println!("{:?}", res);
```
