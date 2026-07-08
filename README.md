# asdot

RFC 5396-compliant Autonomous System Number (ASN) parsing and formatting in Rust.

Supports all three notation formats: ASPLAIN (`65536`), ASDOT (`1.0`), and ASDOT+ (`0.1`).
Display defaults to ASDOT.

```toml
[dependencies]
asdot = "0.1"
```

```rust
use asdot::Asn;

let asn: Asn = "1.0".parse().unwrap();
assert_eq!(asn.value(), 65536);

assert_eq!(asn.to_string(), "1.0");      // ASDOT (default Display)
assert_eq!(asn.to_asplain(), "65536");   // ASPLAIN
assert_eq!(asn.to_asdot_plus(), "1.0"); // ASDOT+
```

<br>

#### License

<sup>
Licensed under the <a href="LICENSE">MIT license</a>.
</sup>
