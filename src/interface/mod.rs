//! This module defines the API for the trade server.
//! Requests are made in a way similar to JSON-RPC 2.0.
//! The only difference between JSON-RPC 2.0 is that the "jsonrpc"
//! string is not included in messages.
//! Read about JSON-RPC here: https://www.jsonrpc.org/specification
use crate::interface::events::{Event, TradeEvent};
use crate::interface::liquidity_pool::LiquidityPoolId;
use crate::interface::order::Order;
use crate::interface::pair::{Pair, PairStateSnapshot};
use crate::interface::requests::{
    CancelOrderRequest, ClearSystemParamRequest, DepositRequest, GrantAccountUserRoleRequest,
    LpTransactRequest, OpenAccountRequest, ReplaceOrderRequest, RevokeAccountUserRoleRequest,
    SetLpParamRequest, SetLpParamsRequest, SetSystemParamRequest, WithdrawRequest,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

pub mod contract_types;
pub mod events;
pub mod liquidity_pool;
pub mod order;
pub mod pair;
pub mod requests;

pub const PRICE_DECIMALS: i64 = 8;
pub const AMOUNT_DECIMALS: i64 = 18;

/// An arbitrary identifier set by the client.
/// An identifier for API messages.
/// Can be used to relate a request to a response.
/// This may either be a request ID (set by the client)
/// or a subscription ID (set by the server).
pub type MessageId = String;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// An optional request ID that the client may set.
    /// If set, the response will contain the same ID so that the client
    /// may collect the response for that particular request.
    pub id: Option<MessageId>,
    #[serde(flatten)]
    pub content: RequestContent,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum RequestContent {
    Subscribe(SubscriptionTopic),
    Unsubscribe(MessageId),
    PlaceOrder(Order),
    CancelOrder(CancelOrderRequest),
    ReplaceOrder(ReplaceOrderRequest),
    Deposit(DepositRequest),
    Withdraw(WithdrawRequest),
    BuyLpToken(LpTransactRequest),
    SellLpToken(LpTransactRequest),
    OpenAccount(OpenAccountRequest),
    SetLpParam(SetLpParamRequest),
    SetLpParams(SetLpParamsRequest),
    SetSystemParam(SetSystemParamRequest),
    ClearSystemParam(ClearSystemParamRequest),
    GrantAccountUserRole(GrantAccountUserRoleRequest),
    RevokeAccountUserRole(RevokeAccountUserRoleRequest),
    GetLpConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "topic", content = "params", rename_all = "camelCase")]
pub enum SubscriptionTopic {
    TradeAccount(AccountId),
    /// Subscribes to LP pair state and tradeability.
    LiquidityPool(LiquidityPoolId),
    /// Subscribes to all trades in the LP.
    LiquidityPoolTrade(LiquidityPoolId),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// For subscriptions, this is the subscription ID.
    /// For all other requests, this is the same as the request ID.
    pub id: Option<MessageId>,
    #[serde(flatten)]
    pub content: ResponseResult<ResponseContent, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum ResponseContent {
    /// Notifies a subscriber of a new publication in the subscription topic.
    Publication(Publication),
    /// Acknowledges the subscription by responding with the subscription ID.
    Subscription(MessageId),
    /// Notifies that the request resulted in an event emission.
    Event(Event),
    /// Notifies that the request resulted in multiple event emissions.
    Events(Vec<Event>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "topic", content = "content", rename_all = "camelCase")]
pub enum Publication {
    TradeAccount(AccountSnapshot),
    Order(Order),
    LpPairTradeability(LpPairPublication<bool>),
    LpPairState(PairStateSnapshot),
    LpTrade(TradeEvent),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LpPairPublication<T> {
    lp_pair: LpPair,
    content: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseResult<T, E> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

impl<T, E> From<Result<T, E>> for ResponseResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        let (result, error) = match value {
            Ok(result) => (Some(result), None),
            Err(error) => (None, Some(error)),
        };
        Self { result, error }
    }
}

impl Response {
    pub fn from<T>(response: Result<ResponseContent, T>, id: Option<MessageId>) -> Self
    where
        T: Debug,
    {
        let result: Result<ResponseContent, String> = match response {
            Ok(result) => Ok(result),
            Err(error) => Err(format!("{:#?}", error)),
        };
        Self {
            id,
            content: result.into(),
        }
    }

    pub fn take(self) -> (Result<ResponseContent, String>, Option<String>) {
        if let Some(result) = self.content.result {
            (Ok(result), self.id)
        } else if let Some(error) = self.content.error {
            (Err(error), self.id)
        } else {
            (Err("invalid response".to_string()), self.id)
        }
    }

    pub fn content(self) -> Result<ResponseContent, String> {
        if let Some(error) = self.content.error {
            return Err(error);
        };
        let Some(content) = self.content.result else {
            return Err("no response content or error received".to_owned());
        };
        Ok(content)
    }
}

impl Request {
    pub fn from(content: RequestContent, id: Option<MessageId>) -> Self {
        Self { id, content }
    }

    pub fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }

    pub fn content(&self) -> &RequestContent {
        &self.content
    }

    pub fn take(self) -> (RequestContent, Option<String>) {
        (self.content, self.id)
    }
}

impl<T> LpPairPublication<T> {
    pub fn new(lp_pair: LpPair, content: T) -> Self {
        Self { lp_pair, content }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AccountRole {
    _None,
    Owner,
    Trader,
    Withdraw,
    Deposit,
    Open,
    ProtocolAdmin,
}

pub type AccountId = u64;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LpPair {
    pub lp_id: LiquidityPoolId,
    pub pair: Pair,
}

impl Display for LpPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.pair, self.lp_id)
    }
}

impl From<(LiquidityPoolId, Pair)> for LpPair {
    fn from((lp_id, pair): (LiquidityPoolId, Pair)) -> Self {
        Self { lp_id, pair }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AccountSnapshot {
    pub id: AccountId,
    pub realized_equity: BigDecimal,
    pub realized_equities_lp: HashMap<LiquidityPoolId, BigDecimal>,
    pub positions: Vec<PositionSnapshot>,
    pub open_orders: Vec<Order>,
    pub lp_profits_withdrawn: LpProfitsWithdrawnSnapshot,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSnapshot {
    pub lp_id: LiquidityPoolId,
    pub pair: Pair,
    pub entry_price: BigDecimal,
    pub size: BigDecimal,
    pub snapshot_sum_fraction_funding: BigDecimal,
    pub snapshot_sum_fraction_borrow: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LpProfitsWithdrawnSnapshot(pub LpIdMap<BigDecimal>);

pub type LpIdMap<T> = HashMap<LiquidityPoolId, T>;
