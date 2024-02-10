use solana_sdk::{pubkey::Pubkey, transaction::Transaction};

//pub mod core;
pub mod anyix;
pub mod route_cache;
pub mod slippage;
pub mod utils;
pub mod price_types;
pub mod quote_types;
pub mod swap_types;
pub mod swapper;

use std::collections::HashMap;

use anyhow::{anyhow, Context};
use reqwest::StatusCode;

use {
    price_types::{format_price_url, PriceResponse},
    quote_types::{format_quote_url, QuoteResponse, RequestOption},
    swap_types::{SwapRequest, SwapResponse, SWAP_BASE},
};

pub struct Client {
    c: reqwest::blocking::Client,
}

impl Client {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            c: reqwest::blocking::ClientBuilder::new()
                .build()?,
        })
    }
    pub fn retrieve_token_list(&self) -> anyhow::Result<HashMap<String, TokenListEntry>> {
        let request = self
            .c
            .get("https://token.jup.ag/all")
            .header("Content-Type", "application/json")
            .build()?;
        Ok(self
            .c
            .execute(request)?
            .json::<Vec<TokenListEntry>>()?
            .into_iter()
            .map(|t| (t.address.clone(), t))
            .collect())
    }
    pub fn new_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        request_options: &[RequestOption<'_>],
    ) -> anyhow::Result<QuoteResponse> {
        let request_url = format_quote_url(input_mint, output_mint, amount, request_options);
        let request = self
            .c
            .get(request_url)
            .header("Content-Type", "application/json")
            .build()?;
        let res = self
            .c
            .execute(request)
            .with_context(|| "failed to execute quote lookup")?;
        if res.status().ne(&StatusCode::OK) {
            return Err(anyhow!("quote lookup failed {}", res.text()?));
        }
        Ok(res
            .json()
            .with_context(|| "failed to decode quote lookup response")?)
    }
    pub fn new_swap(
        &self,
        quote: QuoteResponse,
        user_public_key: &str,
        wrap_unwrap_sol: bool,
    ) -> anyhow::Result<SwapResponse> {
        let req_body = SwapRequest {
            user_public_key: user_public_key.to_string(),
            wrap_and_unwrap_sol: wrap_unwrap_sol,
            quote_response: quote,
            ..Default::default()
        };

        let request = self
            .c
            .post(SWAP_BASE)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .build()?;
        let res = self
            .c
            .execute(request)
            .with_context(|| "failed to execute new_swap")?;
        if res.status().ne(&StatusCode::OK) {
            return Err(anyhow!("new_swap failed {}", res.text()?));
        }
        Ok(res
            .json()
            .with_context(|| "failed to deserialize new_swap response")?)
    }
    pub fn price_query(
        &self,
        input_mint: &str,
        output_mint: &str,
    ) -> anyhow::Result<PriceResponse> {
        let request = self
            .c
            .get(format_price_url(input_mint, output_mint))
            .header("Content-Type", "application/json")
            .build()?;
        let res = self
            .c
            .execute(request)
            .with_context(|| "failed to execute price query")?;
        if res.status().ne(&StatusCode::OK) {
            return Err(anyhow!("price lookup failed {}", res.text()?));
        }
        Ok(res
            .json()
            .with_context(|| "faled to deserialize price query")?)
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenListEntry {
    pub address: String,
    pub chain_id: i64,
    pub decimals: i64,
    pub name: String,
    pub symbol: String,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub tags: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_token_list() {
        let client = Client::new().unwrap();
        let tokens = client.retrieve_token_list().unwrap();
        println!(
            "{:#?}",
            tokens
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .unwrap()
        );
    }
    #[test]
    fn test_jlp_usdc_swap() {
        let client = Client::new().unwrap();

        let response = client
            .new_quote(
                "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                1000000,
                &[RequestOption::SlippageBps(100)],
            )
            .unwrap();

        let response = client
            .new_swap(
                response,
                "5WVCN6gmtCMt61W47aaQ9ByA3Lvfn85ALtTD2VQhLrdx",
                true,
            )
            .unwrap();
        let response = serde_json::to_string_pretty(&response).unwrap();
        //println!("{}", response);

        let price_response = client
            .price_query(
                "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            )
            .unwrap();
        println!("{:#?}", price_response);
    }
}
