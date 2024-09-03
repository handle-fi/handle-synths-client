use crate::environment::{Contracts, ACCOUNT_MESSAGE_SCOPE};
use crate::interface::AccountRole;
use ethers::abi;
use ethers::abi::Token;
use ethers::addressbook::Address;
use ethers::prelude::{LocalWallet, Signature, Signer, U256};
use ethers::utils::{hash_message, keccak256};

#[derive(Clone)]
pub struct User {
    pub signer: LocalWallet,
    pub address: Address,
    pub contracts: Contracts,
}

impl User {
    pub async fn connect(signer: LocalWallet, rpc_url: &str) -> eyre::Result<Self> {
        let address = signer.address();
        let contracts = Contracts::connect(signer.clone(), rpc_url).await?;
        Ok(Self {
            signer,
            address,
            contracts,
        })
    }

    pub fn sign_role_message(
        &self,
        account_id: U256,
        user_nonce: U256,
        role: AccountRole,
    ) -> eyre::Result<Signature> {
        let hash = keccak256(abi::encode(&[
            Token::FixedBytes(ACCOUNT_MESSAGE_SCOPE.clone().into()),
            Token::Uint(U256::from(user_nonce)),
            Token::Uint(U256::from(account_id)),
            Token::Uint(U256::from(role as u8)),
        ]));
        let signature = self.signer.sign_hash(hash_message(hash))?;
        Ok(signature)
    }

    pub async fn get_nonce(&self) -> eyre::Result<U256> {
        let call = &self.contracts.account.user_nonce(self.signer.address());
        call.call().await.map_err(|e| e.into())
    }
}
