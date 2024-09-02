use crate::environment::Contracts;
use crate::interface::contract_types::IERC20;
use ethers::addressbook::Address;
use ethers::prelude::{LocalWallet, Signer, U256};

pub async fn ensure_token_approval(
    contracts: &Contracts,
    signer: &LocalWallet,
    amount: U256,
    token_address: Address,
    target: Address,
) {
    let token = IERC20::new(token_address, contracts.account.client().clone());
    let current_approval = token
        .allowance(signer.address(), target)
        .call()
        .await
        .unwrap();
    if current_approval >= amount {
        return;
    };
    let call = token.approve(target, amount);
    call.send().await.unwrap();
}
