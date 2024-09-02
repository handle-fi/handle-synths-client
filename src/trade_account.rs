use crate::interface::{AccountId, AccountRole, Request, RequestContent, SubscriptionTopic};
use bigdecimal::BigDecimal;
use bigdecimal_ethers_ext::BigDecimalEthersExt;
use ethers::prelude::{Address, LocalWallet, Signer, U256};
use crate::environment::{Contracts, DEPOSIT_TOKEN_DECIMALS, get_user_account_nonce};
use crate::interface::liquidity_pool::LiquidityPoolId;
use crate::interface::requests::DepositRequest;
use crate::trade_account_user::sign_user_role_message;
use crate::utils::ensure_token_approval;

pub struct TradeAccountClient {
    _signer: LocalWallet,
}

impl TradeAccountClient {
    pub async fn open() -> eyre::Result<Self> {
        todo!()
    }

    pub async fn deposit(&self, _amount: BigDecimal) -> eyre::Result<()> {
        todo!()
    }

    pub async fn grant_account_user_role(
        &self,
        _user: Address,
        _role: AccountRole,
    ) -> eyre::Result<()> {
        todo!()
    }
}

async fn get_deposit_ws_message(
    contracts: &Contracts,
    signer: &LocalWallet,
    account_id: AccountId,
    amount: BigDecimal,
    token: Address,
    use_gasless: bool,
) -> String {
    let address = signer.address();
    let nonce = get_user_account_nonce(&contracts, address).await;
    let signature: [u8; 65] =
        sign_user_role_message(signer, U256::from(account_id), nonce, AccountRole::Deposit).into();
    if !use_gasless {
        // May not be required for fxUSD as it is burned/minted by the
        // account contract, therefore no approval required.
        ensure_token_approval(
            contracts,
            signer,
            amount.to_ethers_u256(DEPOSIT_TOKEN_DECIMALS).unwrap(),
            token,
            contracts.account.address(),
        )
            .await;
    }
    let request = Request::from(
        RequestContent::Deposit(DepositRequest {
            amount,
            account_id,
            depositor: address,
            token,
            signature: signature.into(),
            use_gasless: if use_gasless { Some(true) } else { None },
            psm_token: None,
        }),
        None,
    );
    serde_json::to_string(&request).unwrap()
}

fn get_subscribe_trades_message(lp_id_hex: &str) -> String {
    let lp_id = LiquidityPoolId::from_hex_str(lp_id_hex).unwrap();
    let topic = SubscriptionTopic::LiquidityPoolTrade(lp_id);
    let request = Request::from(RequestContent::Subscribe(topic), None);
    serde_json::to_string(&request).unwrap()
}
