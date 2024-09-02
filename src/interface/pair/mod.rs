use crate::interface::liquidity_pool::{OpenInterest, TimestampedBigDecimalMarketSide};
use crate::interface::LpPair;
use ethers::utils::format_bytes32_string;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    io::Write,
    str::FromStr,
};
use thiserror::Error;

pub mod config;

pub const MAX_SYMBOL_LEN: usize = 16;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Pair {
    base_symbol: [u8; MAX_SYMBOL_LEN],
    quote_symbol: [u8; MAX_SYMBOL_LEN],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairStateSnapshot {
    pub lp_pair: LpPair,
    pub sum_fraction_funding: TimestampedBigDecimalMarketSide,
    pub sum_fraction_borrow: TimestampedBigDecimalMarketSide,
    pub open_interest: OpenInterest,
}

#[derive(Debug, Error)]
#[error("failed to create pair: {0}")]
pub struct PairCreationError(Box<dyn std::error::Error + Send + Sync>);

impl Pair {
    pub fn new(base_symbol: &str, quote_symbol: &str) -> Result<Self, PairCreationError> {
        if base_symbol.len() > MAX_SYMBOL_LEN || quote_symbol.len() > MAX_SYMBOL_LEN {
            return Err(PairCreationError("symbol too long".into()));
        }
        let mut pair = Self {
            base_symbol: [0u8; MAX_SYMBOL_LEN],
            quote_symbol: [0u8; MAX_SYMBOL_LEN],
        };
        pair.base_symbol
            .as_mut_slice()
            .write(base_symbol.as_bytes())
            .map_err(|e| PairCreationError(e.into()))?;
        pair.quote_symbol
            .as_mut_slice()
            .write(quote_symbol.as_bytes())
            .map_err(|e| PairCreationError(e.into()))?;
        Ok(pair)
    }

    pub fn get_base_symbol(&self) -> &str {
        let nonzero_part = self
            .base_symbol
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(MAX_SYMBOL_LEN);
        std::str::from_utf8(&self.base_symbol[0..nonzero_part]).unwrap()
    }

    pub fn get_quote_symbol(&self) -> &str {
        let nonzero_part = self
            .quote_symbol
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(MAX_SYMBOL_LEN);
        std::str::from_utf8(&self.quote_symbol[0..nonzero_part]).unwrap()
    }

    pub fn as_bytes32(&self) -> [u8; 32] {
        format_bytes32_string(&self.to_string()).unwrap()
    }
}

impl FromStr for Pair {
    type Err = PairCreationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("/");
        let pair = match (split.next(), split.next()) {
            (Some(base), Some(quote)) => Pair::new(base, quote),
            _ => Err(PairCreationError(
                format!("invalid pair string '{}'", s).into(),
            )),
        };
        if split.next().is_some() {
            return Err(PairCreationError("invalid pair string".into()));
        }
        pair
    }
}

impl Debug for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.get_base_symbol(), self.get_quote_symbol())
    }
}

impl Serialize for Pair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Pair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let pair_string = String::deserialize(deserializer)?;
        Pair::from_str(&pair_string)
            .map_err(|_| serde::de::Error::custom(format!("invalid pair string '{}'", pair_string)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn creation() {
        let pair = Pair::new("ETH", "USD").unwrap();
        assert_eq!(pair.get_base_symbol(), "ETH");
        assert_eq!(pair.get_quote_symbol(), "USD");
        let pair = Pair::new("USD", "CAD").unwrap();
        assert_eq!(pair.get_base_symbol(), "USD");
        assert_eq!(pair.get_quote_symbol(), "CAD");
        let pair = Pair::new("JPY", "USD").unwrap();
        assert_eq!(pair.get_base_symbol(), "JPY");
        assert_eq!(pair.get_quote_symbol(), "USD");
    }

    #[test]
    fn equality() {
        assert_eq!(
            Pair::new("ETH", "USD").unwrap(),
            Pair::new("ETH", "USD").unwrap()
        );
        assert_ne!(
            Pair::new("ETH", "USD").unwrap(),
            Pair::new("AUD", "USD").unwrap()
        );
    }

    #[test]
    fn serialization() {
        let pair = Pair::new("ETH", "USD").unwrap();
        let s = serde_json::to_string(&pair).unwrap();
        assert_eq!(s, "\"ETH/USD\"");
    }

    #[test]
    fn deserialization() {
        let pair_str = "\"ETH/USD\"";
        let pair = serde_json::from_str::<Pair>(pair_str).unwrap();
        assert_eq!(pair, Pair::new("ETH", "USD").unwrap());
    }
}
