use crate::interface::liquidity_pool::LiquidityPoolId;
use crate::interface::{Request, RequestContent, SubscriptionTopic};

pub fn get_subscribe_trades_message(lp_id_hex: &str) -> String {
    let lp_id = LiquidityPoolId::from_hex_str(lp_id_hex).unwrap();
    let topic = SubscriptionTopic::LiquidityPoolTrade(lp_id);
    let request = Request::from(RequestContent::Subscribe(topic), None);
    serde_json::to_string(&request).unwrap()
}
