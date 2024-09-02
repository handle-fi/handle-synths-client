use ethers::addressbook::Address;
use ethers::contract::Lazy;
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Provider, ProviderExt, U256};
use ethers::utils::keccak256;
use futures::stream::{SplitSink, SplitStream};
use pws::{connect_persistent_websocket_async, WsMessageReceiver, WsMessageSender};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use url::Url;
use crate::interface::contract_types::{Account, LiquidityPool, Treasury};

pub const DEPOSIT_TOKEN_DECIMALS: u8 = 18;

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_build_json().unwrap());

pub static ACCOUNT_MESSAGE_SCOPE: Lazy<[u8; 32]> =
    Lazy::new(|| keccak256("HANDLE_SYNTH_ACCOUNT_MESSAGE".as_bytes()));

pub type Client = SignerMiddleware<Provider<Http>, LocalWallet>;


#[derive(Clone)]
pub struct Contracts {
    pub account: Account<Client>,
    pub liquidity_pool: LiquidityPool<Client>,
    pub treasury: Treasury<Client>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub arbitrum_sepolia: NetworkConfig,
    pub arbitrum_one: NetworkConfig,
    pub base: NetworkConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    pub rpc: String,
    pub subgraph: String,
    /// The fxUSD (or equivalent, e.g. mock USD for testnet) token.
    pub usd: Address,
    pub beacon: String,
    pub account: String,
    pub treasury: String,
    pub liquidity_token_factory: String,
    pub liquidity_pool: String,
}

pub type SocketWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type SocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

impl Config {
    pub fn from_build_json() -> serde_json::Result<Self> {
        Ok(serde_json::from_str::<Self>(include_str!(
            "../config.json"
        ))?)
    }
}

impl Contracts {
    pub async fn connect(signer: LocalWallet, rpc_url: &str, network: &str) -> Self {
        let config = get_network_config(network).unwrap();
        let client = get_client(connect_provider(rpc_url).await.unwrap(), signer)
            .await
            .unwrap();
        Self {
            account: Account::new(
                Address::from_str(&config.account).unwrap(),
                client.clone(),
            ),
            liquidity_pool: LiquidityPool::new(
                Address::from_str(&config.liquidity_pool).unwrap(),
                client.clone(),
            ),
            treasury: Treasury::new(Address::from_str(&config.treasury).unwrap(), client),
        }
    }
}

pub async fn connect_websocket(url: Url) -> (WsMessageSender, WsMessageReceiver) {
    let (tx, rx) = connect_persistent_websocket_async(url)
        .await
        .expect("failed to connect");
    (tx, rx)
}

pub async fn get_user_account_nonce(contracts: &Contracts, address: Address) -> U256 {
    let call = contracts.account.user_nonce(address);
    call.call().await.unwrap()
}

pub fn get_network_config(network: &str) -> Option<NetworkConfig> {
    match network.to_lowercase().as_str() {
        "arbitrum-one" => Some(CONFIG.arbitrum_one.clone()),
        "arbitrum-sepolia" => Some(CONFIG.arbitrum_sepolia.clone()),
        "base" => Some(CONFIG.base.clone()),
        _ => None,
    }
}

pub async fn get_client(
    provider: Provider<Http>,
    wallet: LocalWallet,
) -> Result<Arc<Client>, SignerMiddlewareError<Provider<Http>, LocalWallet>> {
    Ok(Arc::new(
        SignerMiddleware::new_with_provider_chain(provider, wallet).await?,
    ))
}

pub async fn connect_provider(rpc_url: &str) -> eyre::Result<Provider<Http>> {
    Ok(Provider::<Http>::try_connect(rpc_url).await?)
}
