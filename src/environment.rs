use crate::interface::contract_types::{Account, LiquidityPool, Treasury};
use ethers::addressbook::Address;
use ethers::contract::Lazy;
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Middleware, Provider, ProviderExt};
use ethers::utils::keccak256;
use eyre::eyre;
use pws::{connect_persistent_websocket_async, WsMessageReceiver, WsMessageSender};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

pub const DEPOSIT_TOKEN_DECIMALS: u8 = 18;

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_build_json().unwrap());

pub static ACCOUNT_MESSAGE_SCOPE: Lazy<[u8; 32]> =
    Lazy::new(|| keccak256("HANDLE_SYNTH_ACCOUNT_MESSAGE".as_bytes()));

pub type Client = SignerMiddleware<Provider<Http>, LocalWallet>;

#[derive(Debug, Clone)]
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
    pub ws: String,
    pub subgraph: String,
    /// The fxUSD (or equivalent, e.g. mock USD for testnet) token.
    pub usd: Address,
    pub beacon: String,
    pub account: String,
    pub treasury: String,
    pub liquidity_token_factory: String,
    pub liquidity_pool: String,
}

impl Config {
    pub fn from_build_json() -> serde_json::Result<Self> {
        Ok(serde_json::from_str::<Self>(include_str!(
            "../config.json"
        ))?)
    }
}

impl Contracts {
    pub async fn connect(signer: LocalWallet, rpc_url: &str) -> eyre::Result<Self> {
        let provider = connect_provider(rpc_url).await?;
        let chain_id = provider.get_chainid().await?.as_u64();
        let Some(config) = get_network_config_by_chain_id(chain_id) else {
            return Err(eyre!("invalid network"));
        };
        let client = get_client(provider, signer).await?;
        Ok(Self {
            account: Account::new(Address::from_str(&config.account)?, client.clone()),
            liquidity_pool: LiquidityPool::new(
                Address::from_str(&config.liquidity_pool)?,
                client.clone(),
            ),
            treasury: Treasury::new(Address::from_str(&config.treasury)?, client),
        })
    }
}

pub async fn connect_websocket(url: Url) -> (WsMessageSender, WsMessageReceiver) {
    let (tx, rx) = connect_persistent_websocket_async(url)
        .await
        .expect("failed to connect");
    (tx, rx)
}

pub fn get_network_config(network: &str) -> Option<NetworkConfig> {
    match network.to_lowercase().as_str() {
        "arbitrum-one" => Some(CONFIG.arbitrum_one.clone()),
        "arbitrum-sepolia" => Some(CONFIG.arbitrum_sepolia.clone()),
        "base" => Some(CONFIG.base.clone()),
        _ => None,
    }
}

pub fn get_network_config_by_chain_id(chain_id: u64) -> Option<NetworkConfig> {
    let Some(network_name) = chain_id_to_network_name(chain_id) else {
        return None;
    };
    get_network_config(&network_name)
}

pub fn chain_id_to_network_name(chain_id: u64) -> Option<String> {
    match chain_id {
        42161 => Some("arbitrum-one".to_owned()),
        421614 => Some("arbitrum-sepolia".to_owned()),
        8453 => Some("base".to_owned()),
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
