use crate::interface::liquidity_pool::LiquidityPoolId;
use crate::interface::pair::Pair;
use crate::interface::requests::TradeSize;
use crate::interface::AccountId;
use bigdecimal::BigDecimal;
use ethers::addressbook::Address;
use ethers::prelude::{Bytes, Signature, H256};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const DEFAULT_EXPIRY_MONTHS: i64 = 3;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    // The order ID, generated as UUIDv7 encoded as u128 (LE).
    // This is not set by users, hence serialization is skipped.
    #[serde(skip_deserializing, default = "new_order_id")]
    pub id: Uuid,
    pub account_id: AccountId,
    pub lp_id: LiquidityPoolId,
    pub size: TradeSize,
    pub pair: Pair,
    pub kind: OrderKind,
    #[serde(skip_deserializing, default)]
    pub status: OrderStatus,
    /// The address that signed the trade request.
    pub account_user: Address,
    /// The order signature.
    /// This includes the user order nonce,
    /// which is an off-chain nonce.
    pub signature: Bytes,
    /// The UUID v4 value for the nonce.
    /// Technically this does not have to be a UUID, it can be any value
    /// not previously used.
    pub nonce: Uuid,
    #[serde(skip_deserializing, default = "now_unix_millis")]
    pub created_timestamp_unix_millis: i64,
    #[serde(default = "default_expiry_unix_millis")]
    pub expiry_timestamp_unix_millis: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(tag = "kind", content = "args", rename_all = "camelCase")]
pub enum OrderKind {
    Market,
    Limit(LimitOrderArgs),
    StopMarket(StopMarketOrderArgs),
    StopLimit(StopLimitOrderArgs),
    LimitTrigger(LimitTriggerOrderArgs),
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrderArgs {
    pub limit_price: BigDecimal,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct StopMarketOrderArgs {
    pub trigger_price: BigDecimal,
    #[serde(default)]
    pub stop_loss: Option<LinkedOrderKind>,
}

#[derive(Copy, Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum LinkedOrderKind {
    Position,
    Order(Uuid),
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct StopLimitOrderArgs {
    pub limit_price: BigDecimal,
    pub trigger_price: BigDecimal,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LimitTriggerOrderArgs {
    pub trigger_price: BigDecimal,
    #[serde(default)]
    pub take_profit: Option<LinkedOrderKind>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(tag = "status", content = "content", rename_all = "camelCase")]
pub enum OrderStatus {
    Open(OrderOpenState),
    Filled(OrderFill),
    Cancelled(OrderCancellation),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum OrderStatusKind {
    Open,
    Filled,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(tag = "status", content = "content", rename_all = "camelCase")]
pub enum OrderOpenState {
    Placed,
    Triggered(OrderTrigger),
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct OrderTrigger {
    pub timestamp_unix_millis: i64,
    pub trigger_price: BigDecimal,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct OrderFill {
    pub price: BigDecimal,
    pub timestamp_unix_millis: i64,
    pub settlement_status: SettlementStatus,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(tag = "status", content = "content", rename_all = "camelCase")]
pub enum SettlementStatus {
    Queued,
    BeforeTx,
    Indexing,
    Settled(OrderSettlement),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct OrderSettlement {
    pub tx_hash: H256,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancellation {
    pub timestamp_unix_millis: i64,
    pub reason: OrderCancellationReason,
    /// The user data for the order cancellation.
    /// Present if cancelled by a user.
    pub user_cancellation: Option<UserOrderCancellation>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct UserOrderCancellation {
    pub nonce: Uuid,
    pub signature: Signature,
    pub address: Address,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum OrderCancellationReason {
    User,
    Expiry,
    FillError,
    ReduceOnlyOrder,
    LinkedOrder,
}

fn now_unix_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn default_expiry_unix_millis() -> i64 {
    now_unix_millis() + (DEFAULT_EXPIRY_MONTHS * 31 * 24 * 60 * 60 * 1_000)
}

fn new_order_id() -> Uuid {
    Uuid::now_v7()
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Open(Default::default())
    }
}

impl Default for OrderOpenState {
    fn default() -> Self {
        Self::Placed
    }
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = OrderStatusKind::from(self);
        let string = kind.to_string();
        write!(f, "{string}")
    }
}

impl From<&OrderStatus> for OrderStatusKind {
    fn from(value: &OrderStatus) -> Self {
        match value {
            OrderStatus::Open(_) => OrderStatusKind::Open,
            OrderStatus::Filled(_) => OrderStatusKind::Filled,
            OrderStatus::Cancelled(_) => OrderStatusKind::Cancelled,
        }
    }
}

impl Display for OrderStatusKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OrderStatusKind::Open => "opn",
            OrderStatusKind::Filled => "fil",
            OrderStatusKind::Cancelled => "cxl",
        };
        write!(f, "{str}")
    }
}

impl TryFrom<&str> for OrderStatusKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let kind = match value {
            "opn" => OrderStatusKind::Open,
            "fil" => OrderStatusKind::Filled,
            "cxl" => OrderStatusKind::Cancelled,
            _ => {
                return Err(format!("invalid status variant: {value}"));
            }
        };
        Ok(kind)
    }
}

impl Display for SettlementStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SettlementStatus::Queued => "qud",
            SettlementStatus::BeforeTx => "btx",
            SettlementStatus::Indexing => "idx",
            SettlementStatus::Settled(_) => "set",
        };
        write!(f, "{str}")
    }
}

impl Display for OrderCancellationReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OrderCancellationReason::User => "usr",
            OrderCancellationReason::Expiry => "exp",
            OrderCancellationReason::FillError => "err",
            OrderCancellationReason::ReduceOnlyOrder => "red",
            OrderCancellationReason::LinkedOrder => "lnk",
        };
        write!(f, "{str}")
    }
}

impl TryFrom<&str> for OrderCancellationReason {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let kind = match value {
            "usr" => OrderCancellationReason::User,
            "exp" => OrderCancellationReason::Expiry,
            "err" => OrderCancellationReason::FillError,
            "red" => OrderCancellationReason::ReduceOnlyOrder,
            "lnk" => OrderCancellationReason::LinkedOrder,
            _ => {
                return Err(format!("invalid cancellation reason variant: {value}"));
            }
        };
        Ok(kind)
    }
}

impl OrderStatus {
    pub fn is_filled(&self) -> bool {
        match self {
            OrderStatus::Filled(_) => true,
            _ => false,
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            OrderStatus::Open(_) => true,
            _ => false,
        }
    }
}

impl StopMarketOrderArgs {
    pub fn is_stop_loss(&self) -> bool {
        self.stop_loss.is_some()
    }

    pub fn linked_stop_loss_order_id(&self) -> Option<Uuid> {
        self.stop_loss.and_then(|link| link.linked_order_id())
    }
}

impl LimitTriggerOrderArgs {
    pub fn is_take_profit(&self) -> bool {
        self.take_profit.is_some()
    }

    pub fn linked_take_profit_order_id(&self) -> Option<Uuid> {
        self.take_profit.and_then(|link| link.linked_order_id())
    }
}

impl LinkedOrderKind {
    pub fn linked_order_id(&self) -> Option<Uuid> {
        match self {
            LinkedOrderKind::Order(id) => Some(*id),
            _ => None,
        }
    }
}
