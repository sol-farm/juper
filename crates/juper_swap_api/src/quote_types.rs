/*
inputMint, outputMint, amount, slippageBps, swapMode, dexes, excludeDexes, onlyDirectRoutes, asLegacyTransaction, platformFeeBps, maxAccounts
*/

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const QUOTE_BASE: &str = "https://quote-api.jup.ag/v6/quote";

/// defines options that can be used to tune the swap request
pub enum RequestOption<'a> {
    SwapMode(SwapMode),
    Dexes(&'a [&'a str]),
    ExcludeDexes(&'a [&'a str]),
    OnlyDirectRoutes,
    AsLegacyTransaction,
    PlatformFeeBps(u64),
    MaxAccounts(usize),
    SlippageBps(i64), // 50 = 0.5%, 100 = 1%, 200 = 2%
}

pub enum SwapMode {
    ExactIn,
    ExactOut,
}

impl ToString for SwapMode {
    fn to_string(&self) -> String {
        if let Self::ExactIn = self {
            "ExactIn".to_string()
        } else {
            "ExactOut".to_string()
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: i64,
    pub platform_fee: Option<PlatformFee>,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
    pub context_slot: i64,
    pub time_taken: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: i64,
}

/// formats the quote url with the required request options
pub fn format_quote_url(
    input_mint: &str,
    output_mint: &str,
    amount: u64,
    request_options: &[RequestOption<'_>],
) -> String {
    let mut quote_url =
        format!("{QUOTE_BASE}?inputMint={input_mint}&outputMint={output_mint}&amount={amount}");
    for request_option in request_options {
        match request_option {
            RequestOption::SwapMode(swap_mode) => {
                quote_url = format!("{quote_url}&swapMode={}", swap_mode.to_string());
            }
            RequestOption::Dexes(dexes) => {
                // skip
            }
            RequestOption::ExcludeDexes(dexes) => {
                // skip
            }
            RequestOption::OnlyDirectRoutes => {
                quote_url = format!("{quote_url}&onlyDirectRoutes=true");
            }
            RequestOption::AsLegacyTransaction => {
                quote_url = format!("{quote_url}&asLegacyTransaction=true");
            }
            RequestOption::PlatformFeeBps(fee_bps) => {
                quote_url = format!("{quote_url}&platformFeeBps={fee_bps}");
            }
            RequestOption::MaxAccounts(max_accounts) => {
                quote_url = format!("{quote_url}&maxAccounts={max_accounts}");
            }
            RequestOption::SlippageBps(slip) => {
                quote_url = format!("{quote_url}&slippageBps={slip}");
            }
        }
    }
    quote_url
}
