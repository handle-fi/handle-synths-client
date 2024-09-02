use crate::interface::liquidity_pool::{LiquidityPoolId, LpTradeEffect, MarketSide};
use crate::interface::order::Order;
use crate::interface::pair::config::PairConfig;
use crate::interface::pair::Pair;
use crate::interface::requests::{
    ClearSystemParamRequest, GrantAccountUserRoleRequest, RevokeAccountUserRoleRequest,
    SetSystemParamRequest,
};
use crate::interface::{AccountId, LpProfitsWithdrawnSnapshot};
use bigdecimal::BigDecimal;
use ethers::prelude::{Address, Bytes};
use serde::{Deserialize, Serialize};

/// System events emitted by the server, sent to an event sink once emitted and
/// potentially forwarded to other systems such as the blockchain for
/// retaining state.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Withdraw(WithdrawEvent),
    Deposit(DepositEvent),
    BuyLpToken(LpTokenBuyEvent),
    SellLpToken(LpTokenSellEvent),
    OpenAccount(OpenAccountEvent),
    SetLpParam(SetLpParamEvent),
    SetLpParams(Vec<SetLpParamEvent>),
    LpConfigUpdate(Vec<LpConfigUpdateEvent>),
    Liquidation(LiquidationEvent),
    LpsFeeWithdraw(LpsFeeWithdrawEvent),
    SetSystemParam(SetSystemParamEvent),
    ClearSystemParam(ClearSystemParamEvent),
    SystemFeeWithdraw(SystemFeeWithdrawEvent),
    GrantAccountUserRole(GrantAccountUserRoleEvent),
    RevokeAccountUserRole(RevokeAccountUserRoleEvent),
    PlaceOrder(Order),
    TriggerOrder(Order),
    FillOrder(FillOrderEvent),
    CancelOrder(Order),
    SettleTrade(TradeEvent),
    ReplaceOrder(ReplaceOrderEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceOrderEvent {
    pub cancelled_order: Order,
    pub new_order: Order,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FillOrderEvent {
    pub trade: TradeEvent,
    pub lp_trade_effect: LpTradeEffect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LpsFeeWithdrawEvent {
    pub recipient: Address,
    pub ids_and_amounts: Vec<(LiquidityPoolId, BigDecimal)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemFeeWithdrawEvent {
    pub token: Address,
    pub recipient: Address,
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TradeEvent {
    pub order: Order,
    /// The fill price.
    pub price: BigDecimal,
    /// The trade size, in lots.
    pub size: BigDecimal,
    /// The realized equity delta of the account after the trade.
    pub realized_pnl: BigDecimal,
    pub margin_fee: BigDecimal,
    /// The funding fee fraction at the time the trade was placed.
    pub sum_fraction_funding: MarketSide<BigDecimal>,
    /// The funding fee fraction at the time the trade was placed.
    pub sum_fraction_borrow: MarketSide<BigDecimal>,
    /// The notional funding rate for LP credit.
    pub lp_funding_rate_notional: BigDecimal,
    pub timestamp_unix_millis: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidationEvent {
    pub trades: Vec<TradeEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawEvent {
    pub account_id: AccountId,
    pub amount: BigDecimal,
    pub timestamp_unix_millis: i64,
    pub account_user: Address,
    pub token: Address,
    pub recipient: Address,
    pub signature: Bytes,
    /// If greater than zero, it means that the user
    /// has withdrawn all of their initial deposit
    /// plus this profit amount.
    pub lp_profits_withdrawn: LpProfitsWithdrawnSnapshot,
    pub psm_token: Option<Address>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DepositEvent {
    pub account_id: AccountId,
    pub amount: BigDecimal,
    pub timestamp_unix_millis: i64,
    pub depositor: Address,
    pub token: Address,
    pub signature: Bytes,
    pub use_gasless: Option<bool>,
    pub psm_token: Option<Address>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenAccountEvent {
    pub account_id: AccountId,
    pub amount: BigDecimal,
    pub timestamp_unix_millis: i64,
    pub owner: Address,
    pub token: Address,
    pub signature: Bytes,
    pub referral_code: Option<String>,
    pub use_gasless: Option<bool>,
    pub psm_token: Option<Address>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LpTokenBuyEvent {
    pub address: Address,
    pub lp_id: LiquidityPoolId,
    pub pay_amount: BigDecimal,
    pub buy_amount: BigDecimal,
    pub timestamp_unix_millis: i64,
    pub signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LpTokenSellEvent {
    pub address: Address,
    pub lp_id: LiquidityPoolId,
    pub pay_amount: BigDecimal,
    pub buy_amount: BigDecimal,
    pub timestamp_unix_millis: i64,
    pub signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetLpParamEvent {
    pub id: LiquidityPoolId,
    // This should be the raw string, it will be utf-8 encoded when sent to the contract.
    pub param_id: String,
    // This is the raw value that will be set in the contract.
    pub value: Bytes,
    pub pool_user: Address,
    pub signature: Bytes,
    pub effect: LpParamEffect,
}

/// An effectful LP param event is required whenever some action needs
/// to occur after changing an LP param.
/// For example, on changing any params related to borrow and funding fees,
/// the existing accrued fees must be charged.
/// The data associated with this effect has to be sent to the blockchain.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LpParamEffect {
    None,
    Borrow(LpParamEffectBorrow),
    Funding(LpParamEffectFunding),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LpParamEffectBorrow {
    /// The funding fee fraction at the time the trade was placed.
    pub sum_fraction_borrow: MarketSide<BigDecimal>,
    pub timestamp_unix_millis: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LpParamEffectFunding {
    /// The funding fee fraction at the time the trade was placed.
    pub sum_fraction_funding: MarketSide<BigDecimal>,
    /// The notional funding rate for LP credit.
    pub lp_funding_rate_notional: BigDecimal,
    pub timestamp_unix_millis: i64,
}

pub type SetSystemParamEvent = SetSystemParamRequest;

pub type ClearSystemParamEvent = ClearSystemParamRequest;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LpConfigUpdateEvent {
    pub lp_id: LiquidityPoolId,
    pub pair: Pair,
    pub config: PairConfig,
}

pub type GrantAccountUserRoleEvent = GrantAccountUserRoleRequest;

pub type RevokeAccountUserRoleEvent = RevokeAccountUserRoleRequest;
