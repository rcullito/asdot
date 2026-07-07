//! RFC 5396-compliant Autonomous System Number (ASN) parsing and formatting.
//!
//! Implements all three notation formats from RFC 5396:
//! - **ASPLAIN**: plain decimal (`65536`)
//! - **ASDOT**: dot notation for 4-byte AS numbers only; plain for 2-byte (`1`, `1.0`)
//! - **ASDOT+**: dot notation always (`0.1`, `1.0`)
//!
//! `Display` defaults to ASDOT notation. Use [`Asn::to_asplain`] or
//! [`Asn::to_asdot_plus`] for the other formats.

use std::fmt;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// The high-order 16-bit word (X in X.Y notation).
    #[inline]
    const fn high(self) -> u16 {
        (self.0 >> 16) as u16
    }

    /// The low-order 16-bit word (Y in X.Y notation).
    #[inline]
    const fn low(self) -> u16 {
        self.0 as u16
    }

    /// Formats in ASPLAIN notation: plain decimal.
    ///
    /// `AS 65536` → `"65536"`
    pub fn to_asplain(self) -> String {
        self.0.to_string()
    }

    /// Formats in ASDOT notation: X.Y only when X > 0, plain decimal otherwise.
    ///
    /// `AS 1` → `"1"`, `AS 65536` → `"1.0"`
    pub fn to_asdot(self) -> String {
        if self.high() > 0 {
            self.to_asdot_plus()
        } else {
            self.low().to_string()
        }
    }

    /// Formats in ASDOT+ notation: always X.Y.
    ///
    /// `AS 1` → `"0.1"`, `AS 65536` → `"1.0"`
    pub fn to_asdot_plus(self) -> String {
        format!("{}.{}", self.high(), self.low())
    }
}

/// `Display` uses ASDOT notation (the crate's namesake and default per this library).
impl fmt::Display for Asn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_asdot())
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAsnError {
    /// The string was empty.
    Empty,
    /// Plain decimal value exceeds `u32::MAX`.
    Overflow,
    /// A dot-notation component (X or Y) exceeded 65535.
    ComponentOverflow,
    /// The string was not valid ASPLAIN, ASDOT, or ASDOT+ notation.
    Invalid,
}

impl fmt::Display for ParseAsnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty string"),
            Self::Overflow => write!(f, "value exceeds maximum ASN (4294967295)"),
            Self::ComponentOverflow => write!(f, "dot-notation component exceeds 65535"),
            Self::Invalid => write!(f, "invalid AS number format"),
        }
    }
}

impl std::error::Error for ParseAsnError {}

impl FromStr for Asn {
    type Err = ParseAsnError;

    /// Parses any RFC 5396 notation: plain decimal (`"65536"`) or dot notation (`"1.0"`, `"0.1"`).
    ///
    /// The parser does not distinguish between ASDOT and ASDOT+ — both produce X.Y strings
    /// and decode identically. The distinction only applies when formatting output.
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

            // Reject multiple dots by trying to parse low_str as u32:
            // "1.2.3" → low_str="2.3" → parse fails → Invalid
            let high: u32 = high_str.parse::<u32>().map_err(|_| ParseAsnError::Invalid)?;
            let low: u32 = low_str.parse::<u32>().map_err(|_| ParseAsnError::Invalid)?;

            if high > 0xFFFF {
                return Err(ParseAsnError::ComponentOverflow);
            }
            if low > 0xFFFF {
                return Err(ParseAsnError::ComponentOverflow);
            }

            Ok(Self((high << 16) | low))
        } else {
            // ASPLAIN: parse as u64 first to detect overflow cleanly
            let value: u64 = s.parse::<u64>().map_err(|_| ParseAsnError::Invalid)?;
            if value > u32::MAX as u64 {
                return Err(ParseAsnError::Overflow);
            }
            Ok(Self(value as u32))
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
        assert_eq!("4294967295".parse::<Asn>().unwrap(), Asn::new(4_294_967_295));
    }

    #[test]
    fn parse_asdot() {
        assert_eq!("0.0".parse::<Asn>().unwrap(), Asn::new(0));
        assert_eq!("0.1".parse::<Asn>().unwrap(), Asn::new(1));
        assert_eq!("0.65535".parse::<Asn>().unwrap(), Asn::new(65535));
        assert_eq!("1.0".parse::<Asn>().unwrap(), Asn::new(65536));
        assert_eq!("1.1".parse::<Asn>().unwrap(), Asn::new(65537));
        assert_eq!("65535.65535".parse::<Asn>().unwrap(), Asn::new(4_294_967_295));
    }

    #[test]
    fn parse_errors() {
        assert_eq!("".parse::<Asn>(), Err(ParseAsnError::Empty));
        assert_eq!("4294967296".parse::<Asn>(), Err(ParseAsnError::Overflow));
        assert_eq!("65536.0".parse::<Asn>(), Err(ParseAsnError::ComponentOverflow));
        assert_eq!("0.65536".parse::<Asn>(), Err(ParseAsnError::ComponentOverflow));
        assert_eq!("abc".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!("1.2.3".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!(".1".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!("1.".parse::<Asn>(), Err(ParseAsnError::Invalid));
        assert_eq!("-1".parse::<Asn>(), Err(ParseAsnError::Invalid));
    }

    // --- Display / formatting ---

    #[test]
    fn display_asdot_default() {
        // 2-byte range: plain decimal
        assert_eq!(Asn::new(0).to_string(), "0");
        assert_eq!(Asn::new(1).to_string(), "1");
        assert_eq!(Asn::new(65535).to_string(), "65535");
        // 4-byte range: X.Y
        assert_eq!(Asn::new(65536).to_string(), "1.0");
        assert_eq!(Asn::new(65537).to_string(), "1.1");
        assert_eq!(Asn::new(4_294_967_295).to_string(), "65535.65535");
    }

    #[test]
    fn to_asplain() {
        assert_eq!(Asn::new(1).to_asplain(), "1");
        assert_eq!(Asn::new(65536).to_asplain(), "65536");
        assert_eq!(Asn::new(4_294_967_295).to_asplain(), "4294967295");
    }

    #[test]
    fn to_asdot_plus() {
        // Always X.Y, even for 2-byte-range ASNs
        assert_eq!(Asn::new(0).to_asdot_plus(), "0.0");
        assert_eq!(Asn::new(1).to_asdot_plus(), "0.1");
        assert_eq!(Asn::new(65535).to_asdot_plus(), "0.65535");
        assert_eq!(Asn::new(65536).to_asdot_plus(), "1.0");
        assert_eq!(Asn::new(4_294_967_295).to_asdot_plus(), "65535.65535");
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
