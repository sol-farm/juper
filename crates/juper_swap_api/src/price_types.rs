use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const PRICE_BASE: &str = "https://price.jup.ag/v4/price";

pub fn format_price_url(mint_to_query: &str, base_mint: &str) -> String {
    format!("{PRICE_BASE}?ids={mint_to_query}&vsToken={base_mint}")
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    pub data: HashMap<String, PriceData>,
    pub time_taken: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub id: String,
    pub mint_symbol: String,
    pub vs_token: String,
    pub vs_token_symbol: String,
    pub price: f64,
}
