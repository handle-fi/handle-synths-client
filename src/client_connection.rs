use crate::environment::{
    SocketWrite, ACCOUNT_MESSAGE_SCOPE
};
use crate::interface::{AccountRole};
use ethers::abi;
use ethers::abi::Token;
use ethers::prelude::{LocalWallet, Signature, Signer, U256};
use ethers::utils::{hash_message, keccak256};
use futures::SinkExt;
use tokio_tungstenite::tungstenite::{Error, Message};

pub struct ClientConnection;

pub async fn send_ws_message<S>(tx: &mut SocketWrite, message: S) -> Result<(), Error>
where
    S: Into<String>,
{
    tx.send(Message::text(message)).await
}

