use crate::client_connection::ClientConnection;
use crate::environment::DEPOSIT_TOKEN_DECIMALS;
use crate::interface::events::Event;
use crate::interface::requests::{DepositRequest, OpenAccountRequest};
use crate::interface::{AccountId, AccountRole, Request, RequestContent, ResponseContent};
use crate::user::User;
use crate::utils::ensure_token_approval;
use bigdecimal::BigDecimal;
use bigdecimal_ethers_ext::BigDecimalEthersExt;
use ethers::prelude::{Address, U256};
use eyre::eyre;

pub struct TradeAccountClient {
    user: User,
    account_id: AccountId,
    connection: ClientConnection,
}

impl TradeAccountClient {
    pub fn new(user: User, account_id: AccountId, connection: ClientConnection) -> Self {
        Self {
            user,
            account_id,
            connection,
        }
    }

    pub async fn open(
        initial_deposit_amount: BigDecimal,
        token: Address,
        use_gasless: bool,
        referral_code: Option<String>,
        user: User,
        connection: ClientConnection,
    ) -> eyre::Result<Self> {
        let message = get_open_account_request(
            &user,
            initial_deposit_amount.clone(),
            token,
            use_gasless,
            referral_code,
        )
        .await?;
        if !use_gasless {
            // TODO: may not be required as it is burned/minted by the
            // account contract, therefore no approval required.
            ensure_token_approval(
                &user.contracts,
                &user.signer,
                initial_deposit_amount
                    .to_ethers_u256(DEPOSIT_TOKEN_DECIMALS)
                    .unwrap(),
                token,
                user.contracts.account.address(),
            )
            .await;
        }
        let response = connection.send_request(message).await?;
        if let Some(error) = response.content.error {
            return Err(eyre!("{error}"));
        };
        let Some(content) = response.content.result else {
            return Err(eyre!("no response content received"));
        };
        let account_id_opt = match content {
            ResponseContent::Event(e) => match e {
                Event::OpenAccount(info) => Some(info.account_id),
                _ => None,
            },
            _ => None,
        };
        let Some(account_id) = account_id_opt else {
            return Err(eyre!("no account received"));
        };
        Ok(Self {
            user,
            account_id,
            connection,
        })
    }

    pub async fn deposit(
        &self,
        amount: BigDecimal,
        token: Address,
        use_gasless: bool,
    ) -> eyre::Result<()> {
        let message = self
            .get_deposit_ws_message(amount.clone(), token, use_gasless)
            .await?;
        if !use_gasless {
            // TODO: may not be required as it is burned/minted by the
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

async fn get_open_account_request(
    user: &User,
    amount: BigDecimal,
    token: Address,
    use_gasless: bool,
    referral_code: Option<String>,
) -> eyre::Result<RequestContent> {
    let nonce = user.get_nonce().await?;
    let signature: [u8; 65] = user
        .sign_role_message(U256::from(0), nonce, AccountRole::Open)?
        .into();
    Ok(RequestContent::OpenAccount(OpenAccountRequest {
        amount,
        token,
        owner: user.address,
        signature: signature.into(),
        referral_code,
        use_gasless: if use_gasless { Some(true) } else { None },
        psm_token: None,
    }))
}
