//! RFC 5396-compliant Autonomous System Number (ASN) parsing and formatting.
//!
//! Implements all three notation formats from RFC 5396:
//! - **ASPLAIN**: plain decimal (`65536`)
//! - **ASDOT**: dot notation for 4-byte AS numbers only; plain for 2-byte (`1`, `1.0`)
//! - **ASDOT+**: dot notation always (`0.1`, `1.0`)
//!
//! `Display` defaults to ASDOT notation. Use [`Asn::to_asplain`] or
//! [`Asn::to_asdot_plus`] for the other formats.
//!
//! # Example
//!
//! ```
//! use asdot::Asn;
//!
//! // Parse any RFC 5396 notation
//! let asn: Asn = "1.0".parse().unwrap();
//! assert_eq!(asn.value(), 65536);
//!
//! // Display defaults to ASDOT
//! assert_eq!(asn.to_string(), "1.0");
//!
//! // Other formats available explicitly
//! assert_eq!(asn.to_asplain(), "65536");
//! assert_eq!(asn.to_asdot_plus(), "1.0");
//! ```

use std::str::FromStr;

/// An Autonomous System Number, stored as a `u32`.
///
/// Parses all three RFC 5396 notation formats and displays in ASDOT by default.
///
/// ```
/// use asdot::Asn;
///
/// let asn: Asn = "1.0".parse().unwrap();
/// assert_eq!(asn.value(), 65536);
/// assert_eq!(asn.to_string(), "1.0");        // ASDOT (default Display)
/// assert_eq!(asn.to_asplain(), "65536");
/// assert_eq!(asn.to_asdot_plus(), "1.0");
///
/// let asn: Asn = "65536".parse().unwrap();   // ASPLAIN input also works
/// assert_eq!(asn.to_string(), "1.0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[display("{}", self.to_asdot())]
pub struct Asn(u32);

impl Asn {
    // --- Well-known values (RFC 6793, RFC 6996, RFC 7607, RFC 5398) ---

    /// AS_TRANS (23456): 2-byte stand-in for 4-byte ASNs in old BGP speakers (RFC 6793).
    pub const TRANS: Self = Self(23456);

    /// The highest valid ASN. 4294967295 is reserved per RFC 7607.
    pub const MAX: Self = Self(4_294_967_294);

    // --- Constructors ---

    /// Creates an `Asn` from a raw `u32`.
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw `u32` value.
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    // --- Notation formatting ---

    // The high-order 16-bit word (X in X.Y notation).
    const fn high(self) -> u16 {
        (self.0 >> 16) as u16
    }

    // The low-order 16-bit word (Y in X.Y notation).
    const fn low(self) -> u16 {
        self.0 as u16
    }

    /// Formats in ASPLAIN notation: plain decimal.
    ///
    /// ```
    /// use asdot::Asn;
    /// assert_eq!(Asn::new(65536).to_asplain(), "65536");
    /// assert_eq!(Asn::new(1).to_asplain(), "1");
    /// ```
    pub fn to_asplain(self) -> String {
        self.0.to_string()
    }

    /// Formats in ASDOT notation: ASDOT+ for 4-byte ASNs, plain decimal for 2-byte ASNs.
    ///
    /// ```
    /// use asdot::Asn;
    /// assert_eq!(Asn::new(1).to_asdot(), "1");
    /// assert_eq!(Asn::new(65536).to_asdot(), "1.0");
    /// ```
    pub fn to_asdot(self) -> String {
        if self.high() > 0 {
            self.to_asdot_plus()
        } else {
            self.low().to_string()
        }
    }

    /// Formats in ASDOT+ notation: always dot notation.
    ///
    /// ```
    /// use asdot::Asn;
    /// assert_eq!(Asn::new(1).to_asdot_plus(), "0.1");
    /// assert_eq!(Asn::new(65536).to_asdot_plus(), "1.0");
    /// ```
    pub fn to_asdot_plus(self) -> String {
        format!("{}.{}", self.high(), self.low())
    }
}

// --- Conversions ---

impl From<u32> for Asn {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u16> for Asn {
    fn from(v: u16) -> Self {
        Self(v as u32)
    }
}

impl From<Asn> for u32 {
    fn from(asn: Asn) -> u32 {
        asn.0
    }
}

// --- Parsing ---

/// Error returned when parsing an ASN string fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseAsnError {
    /// Input was an empty string.
    #[error("empty string")]
    Empty,
    /// Plain decimal value exceeds `u32::MAX` (4294967295).
    #[error("value exceeds maximum ASN (4294967295)")]
    Overflow,
    /// X or Y component in dot notation exceeds 65535.
    #[error("dot-notation component exceeds 65535")]
    ComponentOverflow,
    /// Input is not valid decimal or X.Y notation.
    #[error("invalid AS number format")]
    Invalid,
}

impl FromStr for Asn {
    type Err = ParseAsnError;

    /// Parses any RFC 5396 notation: plain decimal (`"65536"`) or dot notation (`"1.0"`, `"0.1"`).
    ///
    /// The parser does not distinguish between ASDOT and ASDOT+ — both produce X.Y strings
    /// and decode identically. The distinction only applies when formatting output.
    ///
    /// # Errors
    ///
    /// Returns [`ParseAsnError`] if the input is empty, out of range, or not valid notation.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseAsnError::Empty);
        }

        if let Some(dot_pos) = s.find('.') {
            let high_str = &s[..dot_pos];
            let low_str = &s[dot_pos + 1..];

            if high_str.is_empty() || low_str.is_empty() {
                return Err(ParseAsnError::Invalid);
            }

            // Reject multiple dots by trying to parse low_str as u16:
            // "1.2.3" → low_str="2.3" → parse fails → Invalid
            let high: u16 = high_str.parse::<u16>().map_err(|e| match e.kind() {
                std::num::IntErrorKind::PosOverflow => ParseAsnError::ComponentOverflow,
                _ => ParseAsnError::Invalid,
            })?;
            let low: u16 = low_str.parse::<u16>().map_err(|e| match e.kind() {
                std::num::IntErrorKind::PosOverflow => ParseAsnError::ComponentOverflow,
                _ => ParseAsnError::Invalid,
            })?;

            // since we shifted high all the way to the left, bitwise or just tasks the lower bits on what would presumably be all zeroes
            Ok(Self(((high as u32) << 16) | low as u32))
        } else {
            let value: u32 = s.parse::<u32>().map_err(|e| match e.kind() {
                std::num::IntErrorKind::PosOverflow => ParseAsnError::Overflow,
                _ => ParseAsnError::Invalid,
            })?;
            Ok(Self(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Parsing ---

    #[test]
    fn parse_asplain() {
        assert_eq!("0".parse::<Asn>().unwrap(), Asn::new(0));
        assert_eq!("65535".parse::<Asn>().unwrap(), Asn::new(65535));
        assert_eq!("65536".parse::<Asn>().unwrap(), Asn::new(65536));
        assert_eq!(
            "4294967295".parse::<Asn>().unwrap(),
            Asn::new(4_294_967_295)
        );
    }

    #[test]
    fn parse_asdot() {
        assert_eq!("0.0".parse::<Asn>().unwrap(), Asn::new(0));
        assert_eq!("0.1".parse::<Asn>().unwrap(), Asn::new(1));
        assert_eq!("0.65535".parse::<Asn>().unwrap(), Asn::new(65535));
        assert_eq!("1.0".parse::<Asn>().unwrap(), Asn::new(65536));
        assert_eq!("1.1".parse::<Asn>().unwrap(), Asn::new(65537));
        assert_eq!(
            "65535.65535".parse::<Asn>().unwrap(),
            Asn::new(4_294_967_295)
        );
    }

    #[test]
    fn asplain_overflow() {
        assert_eq!("4294967296".parse::<Asn>(), Err(ParseAsnError::Overflow));
        assert_eq!("99999999999".parse::<Asn>(), Err(ParseAsnError::Overflow));
    }

    #[test]
    fn asplain_invalid() {
        assert_eq!("abc".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!("-1".parse::<Asn>(), Err(ParseAsnError::Invalid));
    }

    #[test]
    fn parse_errors() {
        assert_eq!("".parse::<Asn>(), Err(ParseAsnError::Empty));
        assert_eq!(
            "65536.0".parse::<Asn>(),
            Err(ParseAsnError::ComponentOverflow)
        );
        assert_eq!(
            "0.65536".parse::<Asn>(),
            Err(ParseAsnError::ComponentOverflow)
        );
        assert_eq!("1.2.3".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!(".1".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!("1.".parse::<Asn>(), Err(ParseAsnError::Invalid));
    }

    // --- Round-trip ---

    #[test]
    fn roundtrip_asplain() {
        for v in [0u32, 1, 65535, 65536, 100_000, 4_294_967_295] {
            let asn = Asn::new(v);
            assert_eq!(asn.to_asplain().parse::<Asn>().unwrap(), asn);
        }
    }

    #[test]
    fn roundtrip_asdot() {
        for v in [0u32, 1, 65535, 65536, 65537, 4_294_967_295] {
            let asn = Asn::new(v);
            assert_eq!(asn.to_asdot().parse::<Asn>().unwrap(), asn);
        }
    }

    #[test]
    fn roundtrip_asdot_plus() {
        for v in [0u32, 1, 65535, 65536, 65537, 4_294_967_295] {
            let asn = Asn::new(v);
            assert_eq!(asn.to_asdot_plus().parse::<Asn>().unwrap(), asn);
        }
    }

    // --- Conversions ---

    #[test]
    fn conversions() {
        assert_eq!(Asn::from(65536u32), Asn::new(65536));
        assert_eq!(Asn::from(1u16), Asn::new(1));
        assert_eq!(u32::from(Asn::new(65536)), 65536u32);
    }
}
