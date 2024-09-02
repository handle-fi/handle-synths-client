use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct PairConfig {
    pub initial_margin_fraction: BigDecimal,
    pub maintenance_margin_fraction: BigDecimal,
    pub incremental_initial_margin_fraction: BigDecimal,
    pub baseline_position_size: BigDecimal,
    pub incremental_position_size: BigDecimal,
    pub margin_fee_fraction: BigDecimal,
    pub symmetrical_spread_fraction: BigDecimal,
    pub is_active: bool,
    /// Whether this market only allow position decreases.
    /// If so, users may close or reduce positions but not open or increase.
    pub is_reduce_only: bool,
    // The maximum allowed difference between longs and shorts.
    pub max_open_interest_diff: Option<BigDecimal>,
    pub max_open_interest_long: Option<BigDecimal>,
    pub max_open_interest_short: Option<BigDecimal>,
    // From GLP borrow fee model.
    pub borrow_fee_factor: BigDecimal,
    // From https://github.com/gmx-io/gmx-synthetics#funding-fees
    pub funding_factor: BigDecimal,
    pub funding_exponent: BigDecimal,
    /// One a price is received, traders can trade with that price for
    /// `trade_price_expiration` seconds.
    pub trade_price_expiration: Option<u64>,
    pub use_price_impact: bool,
    pub price_impact_fraction: Option<BigDecimal>,
    pub skew_scale: Option<BigDecimal>,
}
