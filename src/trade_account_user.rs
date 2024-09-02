use ethers::abi;
use ethers::abi::Token;
use ethers::prelude::{LocalWallet, Signature, U256};
use ethers::utils::{hash_message, keccak256};
use crate::environment::ACCOUNT_MESSAGE_SCOPE;
use crate::interface::AccountRole;

pub struct TradeAccountUser;

pub fn sign_user_role_message(
    signer: &LocalWallet,
    account_id: U256,
    user_nonce: U256,
    role: AccountRole,
) -> Signature {
    let hash = keccak256(abi::encode(&[
        Token::FixedBytes(ACCOUNT_MESSAGE_SCOPE.clone().into()),
        Token::Uint(U256::from(user_nonce)),
        Token::Uint(U256::from(account_id)),
        Token::Uint(U256::from(role as u8)),
    ]));
    signer.sign_hash(hash_message(hash)).unwrap()
}
