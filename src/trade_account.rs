use crate::client_connection::ClientConnection;
use crate::environment::DEPOSIT_TOKEN_DECIMALS;
use crate::interface::events::Event;
use crate::interface::requests::{DepositRequest, GrantAccountUserRoleRequest, OpenAccountRequest};
use crate::interface::{AccountId, AccountRole, RequestContent, ResponseContent};
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
    pub fn from_existing(account_id: AccountId, user: User, connection: ClientConnection) -> Self {
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
        let request = get_open_account_request(
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
        let response = connection.send_request(request).await?;
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
        let request = self
            .get_deposit_ws_request(amount.clone(), token, use_gasless)
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
        self.connection.send_request(request).await?;
        Ok(())
    }

    pub async fn grant_account_user_role(
        &self,
        user: Address,
        role: AccountRole,
    ) -> eyre::Result<()> {
        let request = self.get_grant_role_request(user, role).await?;
        self.connection.send_request(request).await?;
        Ok(())
    }

    async fn get_deposit_ws_request(
        &self,
        amount: BigDecimal,
        token: Address,
        use_gasless: bool,
    ) -> eyre::Result<RequestContent> {
        let nonce = self.user.get_nonce().await?;
        let signature: [u8; 65] = self
            .user
            .sign_role_message(U256::from(self.account_id), nonce, AccountRole::Deposit)?
            .into();
        Ok(RequestContent::Deposit(DepositRequest {
            amount,
            account_id: self.account_id,
            depositor: self.user.address,
            token,
            signature: signature.into(),
            use_gasless: if use_gasless { Some(true) } else { None },
            psm_token: None,
        }))
    }

    async fn get_grant_role_request(
        &self,
        user: Address,
        role: AccountRole,
    ) -> eyre::Result<RequestContent> {
        let nonce = self.user.get_nonce().await?;
        let signature: [u8; 65] = self
            .user
            .sign_role_message(U256::from(self.account_id), nonce, AccountRole::Owner)?
            .into();
        Ok(RequestContent::GrantAccountUserRole(
            GrantAccountUserRoleRequest {
                account_id: self.account_id,
                user,
                role,
                account_owner: self.user.address,
                owner_signature: signature.into(),
            },
        ))
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::environment::CONFIG;
    use ethers::prelude::LocalWallet;
    use std::env;

    #[tokio::test]
    async fn test_account() {
        dotenv::dotenv().unwrap();
        let wallet = env::var("TEST_PRIVATE_KEY")
            .unwrap()
            .parse::<LocalWallet>()
            .unwrap();
        let rpc_url = env::var("TEST_RPC_URL").unwrap();
        let initial_deposit = BigDecimal::from(1);
        let deposit_token = CONFIG.arbitrum_sepolia.usd;
        let ws_url = &CONFIG.arbitrum_sepolia.ws;
        let user = User::connect(wallet, &rpc_url).await.unwrap();
        let connection = ClientConnection::connect(ws_url).await.unwrap();
        let account =
            TradeAccountClient::open(initial_deposit, deposit_token, true, None, user, connection)
                .await
                .unwrap();
        assert!(account.account_id > 0);
    }
}
