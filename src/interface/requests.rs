use crate::interface::liquidity_pool::LiquidityPoolId;
use crate::interface::order::Order;
use crate::interface::{AccountId, AccountRole};
use bigdecimal::BigDecimal;
use ethers::addressbook::Address;
use ethers::prelude::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A trade size, which may be specified in either lots or LPC.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(tag = "unit", content = "amount", rename_all = "camelCase")]
pub enum TradeSize {
    /// Size specified in lots.
    Lot(BigDecimal),
    /// Size specified in the LP currency, e.g. USD.
    Lpc(BigDecimal),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderRequest {
    pub account_id: AccountId,
    pub account_user: Address,
    pub order_id: Uuid,
    pub nonce: Uuid,
    pub signature: Bytes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceOrderRequest {
    pub account_id: AccountId,
    pub account_user: Address,
    pub order_id: Uuid,
    pub nonce: Uuid,
    pub cancel_signature: Bytes,
    pub new_order: Order,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositRequest {
    pub amount: BigDecimal,
    pub account_id: AccountId,
    pub depositor: Address,
    pub token: Address,
    pub signature: Bytes,
    pub use_gasless: Option<bool>,
    /// The token in the first leg of the deposit.
    /// If present, it is swapped to the `token` parameter
    /// via the hPSM.
    pub psm_token: Option<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenAccountRequest {
    pub amount: BigDecimal,
    // TODO: add recipient & depositor field, remove "owner".
    pub owner: Address,
    pub token: Address,
    pub signature: Bytes,
    pub referral_code: Option<String>,
    pub use_gasless: Option<bool>,
    pub psm_token: Option<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawRequest {
    pub amount: BigDecimal,
    pub account_id: AccountId,
    /// The address that signed the withdrawal request.
    pub account_user: Address,
    pub token: Address,
    pub recipient: Address,
    pub signature: Bytes,
    pub psm_token: Option<Address>,
}

/// LpTransact refers to buying or selling liquidity tokens.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LpTransactRequest {
    pub lp_id: LiquidityPoolId,
    pub amount: BigDecimal,
    pub address: Address,
    pub signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetLpParamRequest {
    pub id: LiquidityPoolId,
    // This should be the raw string, it will be utf-8 encoded when sent to the contract.
    pub param_id: String,
    // This is the raw value that will be set in the contract.
    pub value: Bytes,
    pub pool_user: Address,
    pub signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetLpParamsRequest {
    pub id: LiquidityPoolId,
    pub pool_user: Address,
    pub params: Vec<LpParamRequest>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LpParamRequest {
    // This should be the raw string, it will be utf-8 encoded when sent to the contract.
    pub param_id: String,
    // This is the raw value that will be set in the contract.
    pub value: Bytes,
    pub signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetSystemParamRequest {
    pub param_id: String,
    pub param_value: Bytes,
    pub admin_request: AdminRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearSystemParamRequest {
    pub param_id: String,
    pub admin_request: AdminRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GrantAccountUserRoleRequest {
    pub account_id: AccountId,
    pub user: Address,
    pub role: AccountRole,
    pub account_owner: Address,
    pub owner_signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RevokeAccountUserRoleRequest {
    pub account_id: AccountId,
    pub user: Address,
    pub role: AccountRole,
    pub account_owner: Address,
    pub owner_signature: Bytes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdminRequest {
    pub trade_account_id: AccountId,
    pub admin: Address,
    pub signature: Bytes,
}
