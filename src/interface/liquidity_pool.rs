use crate::interface::pair::Pair;
use bigdecimal::BigDecimal;
use ethers::abi::ethereum_types::FromStrRadixErr;
use ethers::abi::AbiEncode;
use ethers::types::{Address, U256};
use ethers::utils::hex;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Eq, PartialEq, Hash, Copy, Deserialize, Default)]
pub struct LiquidityPoolId(U256);

impl LiquidityPoolId {
    pub fn new(id: U256) -> Self {
        Self(id)
    }

    pub fn as_u256(&self) -> U256 {
        self.0
    }

    pub fn to_address(&self) -> Address {
        // 12 bytes are skipped since U256 is 32 bytes but address is 20.
        let address_bytes = self.0.encode().into_iter().skip(12).collect::<Vec<u8>>();
        Address::from_slice(&address_bytes)
    }

    pub fn from_hex_str(str: &str) -> Result<Self, FromStrRadixErr> {
        let u256 = U256::from_str_radix(str, 16)?;
        Ok(Self::new(u256))
    }

    /// Creates a new LiquidityPoolId from big endian bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let id = U256::from_big_endian(&bytes);
        Self(id)
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_address_string())
    }

    fn to_address_string(&self) -> String {
        format!("0x{}", hex::encode(self.to_address().as_bytes()))
    }

    /// Returns the underlying U256 bytes as big-endian.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.encode()
    }
}

impl Display for LiquidityPoolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Debug for LiquidityPoolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Serialize for LiquidityPoolId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_address_string())
    }
}

/// This is used to store the effect a trade has on a liquidity pool.
/// It can be thought of as an operation that is applied to the liquidity pool.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LpTradeEffect {
    pub lp_id: LiquidityPoolId,
    pub pair: Pair,
    pub realized_equity: BigDecimal,
    pub old_size: BigDecimal,
    pub next_size: BigDecimal,
    /// The funding fee fraction at the time the trade was placed.
    pub sum_fraction_funding: MarketSide<BigDecimal>,
    /// The funding fee fraction at the time the trade was placed.
    pub sum_fraction_borrow: MarketSide<BigDecimal>,
    /// The notional funding rate for LP credit.
    pub lp_funding_rate_notional: BigDecimal,
    pub timestamp_unix_millis: i64,
}

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MarketSide<T: Default> {
    pub long: T,
    pub short: T,
}

pub type OpenInterest = MarketSide<BigDecimal>;

pub type TimestampedBigDecimalMarketSide = TimestampedValue<MarketSide<BigDecimal>>;

pub type TimestampedBigDecimal = TimestampedValue<BigDecimal>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimestampedValue<T> {
    pub value: T,
    pub timestamp: i64,
}

impl<T> TimestampedValue<T> {
    pub fn new(value: T, timestamp: i64) -> Self {
        Self { value, timestamp }
    }
}

impl TimestampedBigDecimalMarketSide {
    pub fn long(&self) -> TimestampedBigDecimal {
        TimestampedBigDecimal::new(self.value.long.clone(), self.timestamp)
    }

    pub fn short(&self) -> TimestampedBigDecimal {
        TimestampedBigDecimal::new(self.value.short.clone(), self.timestamp)
    }
}
