use crate::client_connection::ClientConnection;
use crate::environment::DEPOSIT_TOKEN_DECIMALS;
use crate::interface::events::{DepositEvent, Event, GrantAccountUserRoleEvent};
use crate::interface::requests::{DepositRequest, GrantAccountUserRoleRequest, OpenAccountRequest};
use crate::interface::{AccountId, AccountRole, RequestContent, ResponseContent};
use crate::user::User;
use crate::utils::ensure_token_approval;
use bigdecimal::BigDecimal;
use bigdecimal_ethers_ext::BigDecimalEthersExt;
use ethers::prelude::{Address, U256};
use eyre::eyre;

pub struct TradeAccountClient {
    id: AccountId,
    user: User,
    connection: ClientConnection,
}

impl TradeAccountClient {
    pub fn from_existing(account_id: AccountId, user: User, connection: ClientConnection) -> Self {
        Self {
            id: account_id,
            user,
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
        let content = response.content().map_err(|e| eyre!(e))?;
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
            id: account_id,
            user,
            connection,
        })
    }

    pub async fn deposit(
        &self,
        amount: BigDecimal,
        token: Address,
        use_gasless: bool,
    ) -> eyre::Result<DepositEvent> {
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
        let response = self.connection.send_request(request).await?;
        let content = response.content().map_err(|e| eyre!(e))?;
        let deposit_event_opt = match &content {
            ResponseContent::Event(e) => match e {
                Event::Deposit(e) => Some(e),
                _ => None,
            },
            _ => None,
        };
        let Some(deposit_event) = deposit_event_opt else {
            return Err(eyre!("did not receive deposit event; {content:#?}"));
        };
        Ok(deposit_event.clone())
    }

    pub async fn grant_account_user_role(
        &self,
        user: Address,
        role: AccountRole,
    ) -> eyre::Result<GrantAccountUserRoleEvent> {
        let request = self.get_grant_role_request(user, role).await?;
        let response = self.connection.send_request(request).await?;
        let content = response.content().map_err(|e| eyre!(e))?;
        let grant_role_event_opt = match &content {
            ResponseContent::Event(e) => match e {
                Event::GrantAccountUserRole(e) => Some(e),
                _ => None,
            },
            _ => None,
        };
        let Some(grant_role_event) = grant_role_event_opt else {
            return Err(eyre!("did not receive grant role event; {content:#?}"));
        };
        Ok(grant_role_event.clone())
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
            .sign_role_message(U256::from(self.id), nonce, AccountRole::Deposit)?
            .into();
        Ok(RequestContent::Deposit(DepositRequest {
            amount,
            account_id: self.id,
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
            .sign_role_message(U256::from(self.id), nonce, AccountRole::Owner)?
            .into();
        Ok(RequestContent::GrantAccountUserRole(
            GrantAccountUserRoleRequest {
                account_id: self.id,
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
    use ethers::prelude::{LocalWallet, H160};
    use std::env;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_account() {
        _ = dotenv::dotenv();
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
        // Given the user has enough funds to initially deposit to an account;
        // When an account is requested to be opened;
        let account =
            TradeAccountClient::open(initial_deposit, deposit_token, true, None, user, connection)
                .await
                .unwrap();
        // Then the account should have been opened;
        assert!(account.id > 0);
        // Given te user has enough funds to perform a deposit;
        // When the deposit is requested;
        let deposit_event = account
            .deposit(BigDecimal::from(1), deposit_token, true)
            .await
            .unwrap();
        // Then the account should have been deposited to;
        assert_eq!(deposit_event.account_id, account.id);
        assert_eq!(deposit_event.amount, BigDecimal::from(1));
        assert_eq!(deposit_event.token, deposit_token);
        // When an account requests to grant the trade role to an address;
        let trade_role_recipient =
            H160::from_str("00000000000000000000000000000000deadbeef").unwrap();
        let grant_role_event = account
            .grant_account_user_role(trade_role_recipient, AccountRole::Trader)
            .await
            .unwrap();
        // Then the response should be successful;
        assert_eq!(grant_role_event.role, AccountRole::Trader);
        assert_eq!(grant_role_event.user, trade_role_recipient);
        assert_eq!(grant_role_event.account_id, account.id);
        assert_eq!(grant_role_event.account_owner, account.user.address);
    }
}
