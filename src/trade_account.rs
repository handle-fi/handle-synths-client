use crate::client_connection::ClientConnection;
use crate::environment::DEPOSIT_TOKEN_DECIMALS;
use crate::interface::requests::DepositRequest;
use crate::interface::{AccountId, AccountRole, Request, RequestContent};
use crate::user::User;
use crate::utils::ensure_token_approval;
use bigdecimal::BigDecimal;
use bigdecimal_ethers_ext::BigDecimalEthersExt;
use ethers::prelude::{Address, U256};

pub struct TradeAccountClient {
    user: User,
    account_id: AccountId,
    connection: ClientConnection,
}

impl TradeAccountClient {
    pub async fn open() -> eyre::Result<Self> {
        todo!()
    }

    pub async fn connect(account_id: AccountId) -> eyre::Result<Self> {
        todo!()
    }

    pub async fn deposit(
        &self,
        amount: BigDecimal,
        token: Address,
        use_gasless: bool,
    ) -> eyre::Result<()> {
        let message = self
            .get_deposit_ws_message(amount, token, use_gasless)
            .await?;
        self.connection.send_ws_message(message).await?;
        Ok(())
    }

    pub async fn grant_account_user_role(
        &self,
        _user: Address,
        _role: AccountRole,
    ) -> eyre::Result<()> {
        todo!()
    }

    async fn get_deposit_ws_message(
        &self,
        amount: BigDecimal,
        token: Address,
        use_gasless: bool,
    ) -> eyre::Result<String> {
        let nonce = self.user.get_nonce().await?;
        let signature: [u8; 65] = self
            .user
            .sign_role_message(U256::from(self.account_id), nonce, AccountRole::Deposit)?
            .into();
        if !use_gasless {
            // May not be required for fxUSD as it is burned/minted by the
            // account contract, therefore no approval required.
            ensure_token_approval(
                &self.user.contracts,
                &self.user.signer,
                amount.to_ethers_u256(DEPOSIT_TOKEN_DECIMALS).unwrap(),
                token,
                self.user.contracts.account.address(),
            )
            .await;
        }
        let request = Request::from(
            RequestContent::Deposit(DepositRequest {
                amount,
                account_id: self.account_id,
                depositor: self.user.address,
                token,
                signature: signature.into(),
                use_gasless: if use_gasless { Some(true) } else { None },
                psm_token: None,
            }),
            None,
        );
        Ok(serde_json::to_string(&request)?)
    }
}
