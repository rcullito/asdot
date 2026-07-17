//! RFC 5396-compliant Autonomous System Number (ASN) parsing and formatting.
//! ASNs identify the networks that exchange routes with BGP.
//!
//! Implements all three notation formats from RFC 5396:
//! - **ASPLAIN**: plain decimal (`65536`)
//! - **ASDOT**: dot notation for 4-byte AS numbers only; plain for 2-byte (`1`, `1.0`)
//! - **ASDOT+**: dot notation always (`0.1`, `1.0`)
//!
//! `Display` formats in ASDOT notation. For ASPLAIN output, format the raw
//! value from [`Asn::value`].
//!
//! # Example
//!
//! ```
//! use asdot::Asn;
//!
//! // Parse any RFC 5396 notation
//! let asn: Asn = "1.0".parse()?;
//! assert_eq!(asn.value(), 65536);
//!
//! // Display formats as ASDOT
//! assert_eq!(asn.to_string(), "1.0");
//!
//! // ASPLAIN via the raw value
//! assert_eq!(asn.value().to_string(), "65536");
//! # Ok::<(), asdot::ParseAsnError>(())
//! ```

#![cfg_attr(not(test), no_std)]

use core::fmt;
use core::num::IntErrorKind;
use core::str::FromStr;

/// An Autonomous System Number, stored as a `u32`.
///
/// Parses all three RFC 5396 notation formats and displays in ASDOT.
///
/// ```
/// use asdot::Asn;
///
/// let asn: Asn = "1.0".parse()?;
/// assert_eq!(asn.value(), 65536);
/// assert_eq!(asn.to_string(), "1.0");        // ASDOT Display
///
/// let asn: Asn = "65536".parse()?;           // ASPLAIN input also works
/// assert_eq!(asn.to_string(), "1.0");
/// # Ok::<(), asdot::ParseAsnError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Asn(u32);

impl Asn {
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
    const fn low(self) -> u32 {
        self.0 & 0xFFFF
    }
}

impl fmt::Display for Asn {
    /// Formats in ASDOT notation: dot notation for 4-byte ASNs, plain decimal for 2-byte ASNs.
    ///
    /// ```
    /// use asdot::Asn;
    /// assert_eq!(Asn::new(1).to_string(), "1");
    /// assert_eq!(Asn::new(65536).to_string(), "1.0");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.high() > 0 {
            write!(f, "{}.{}", self.high(), self.low())
        } else {
            write!(f, "{}", self.low())
        }
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
    /// Input was an empty string.
    Empty,
    /// Plain decimal value exceeds `u32::MAX` (4294967295).
    Overflow,
    /// X or Y component in dot notation exceeds 65535.
    ComponentOverflow,
    /// Input is not valid decimal or X.Y notation.
    Invalid,
}

impl fmt::Display for ParseAsnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty string",
            Self::Overflow => "value exceeds maximum ASN (4294967295)",
            Self::ComponentOverflow => "dot-notation component exceeds 65535",
            Self::Invalid => "invalid AS number format",
        })
    }
}

impl core::error::Error for ParseAsnError {}

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
                IntErrorKind::PosOverflow => ParseAsnError::ComponentOverflow,
                _ => ParseAsnError::Invalid,
            })?;
            let low: u16 = low_str.parse::<u16>().map_err(|e| match e.kind() {
                IntErrorKind::PosOverflow => ParseAsnError::ComponentOverflow,
                _ => ParseAsnError::Invalid,
            })?;

            // since we shifted high all the way to the left, bitwise or just tasks the lower bits on what would presumably be all zeroes
            Ok(Self(((high as u32) << 16) | low as u32))
        } else {
            let value: u32 = s.parse::<u32>().map_err(|e| match e.kind() {
                IntErrorKind::PosOverflow => ParseAsnError::Overflow,
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

    // --- Display ---

    #[test]
    fn display_asdot() {
        assert_eq!(Asn::new(1).to_string(), "1");
        assert_eq!(Asn::new(65535).to_string(), "65535");
        assert_eq!(Asn::new(65536).to_string(), "1.0");
        assert_eq!(Asn::new(4_294_967_295).to_string(), "65535.65535");
    }

    // --- Round-trip ---

    #[test]
    fn roundtrip_display() {
        for v in [0u32, 1, 65535, 65536, 65537, 100_000, 4_294_967_295] {
            let asn = Asn::new(v);
            assert_eq!(asn.to_string().parse::<Asn>().unwrap(), asn);
        }
    }

    #[test]
    fn roundtrip_value_display() {
        for v in [0u32, 1, 65535, 65536, 100_000, 4_294_967_295] {
            let asn = Asn::new(v);
            assert_eq!(asn.value().to_string().parse::<Asn>().unwrap(), asn);
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
