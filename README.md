# asdot

[<img alt="github" src="https://img.shields.io/badge/github-rcullito/asdot-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rcullito/asdot)
[<img alt="crates.io" src="https://img.shields.io/crates/v/asdot.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/asdot)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-asdot-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/asdot)

RFC 5396-compliant Autonomous System Number (ASN) parsing and formatting in Rust.
ASNs identify the networks that exchange routes with BGP.

Parses all three notation formats: ASPLAIN (`65536`), ASDOT (`1.0`), and ASDOT+ (`0.1`).
`Display` formats in ASDOT; for ASPLAIN output, format the raw `value()`.

```toml
[dependencies]
asdot = "0.4"
```

```rust
use asdot::{Asn, ParseAsnError};

fn main() -> Result<(), ParseAsnError> {
    let asn: Asn = "1.0".parse()?;
    assert_eq!(asn.value(), 65536);

    assert_eq!(asn.to_string(), "1.0");             // ASDOT (default Display)
    assert_eq!(asn.value().to_string(), "65536");   // ASPLAIN via the raw value
    Ok(())
}
```

Enable the `serde` feature to derive `Serialize`/`Deserialize` on `Asn`:

```toml
[dependencies]
asdot = { version = "0.4", features = ["serde"] }
```

<br>

#### License

<sup>
Licensed under the <a href="LICENSE">MIT license</a>.
</sup>
