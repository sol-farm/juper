use std::collections::HashMap;

use anyhow::anyhow;
use async_trait::async_trait;

use crate::{
    error::maybe_jupiter_api_error,
    types::{IndexedRouteMap, Price, RouteMap, Swap, SwapConfig, SwapResponse},
};
use crate::{
    slippage::{FeeBps, Slippage},
    Quote, Response,
};
use serde;
use solana_sdk::pubkey::Pubkey;

use once_cell::sync::Lazy;
pub static REQ_CLIENT: Lazy<reqwest::blocking::Client> =
    Lazy::new(|| reqwest::blocking::Client::builder().build().unwrap());
pub static ASYNC_REQ_CLIENT: Lazy<reqwest::Client> =
    Lazy::new(|| reqwest::Client::builder().build().unwrap());

#[derive(Clone, Copy, PartialEq, Eq)]
/// The API type provides variants which implements the JupAPI trait
/// for the corresponding api version as indicated by the variant
pub enum API {
    V1,
    V4,
    V6,
}

impl API {
    pub fn client(&self) -> reqwest::blocking::Client {
        REQ_CLIENT.clone()
    }
    pub fn async_client(&self) -> reqwest::Client {
        ASYNC_REQ_CLIENT.clone()
    }
}

pub type MarketCaches = Vec<MarketCacheAccount>;

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCacheAccount {
    pub data: Vec<String>,
    pub executable: bool,
    pub lamports: i64,
    pub owner: String,
    pub rent_epoch: i64,
    pub pubkey: String,
}

#[async_trait]
pub trait JupAPI {
    /// returns a url string used to submit a route map request to the api
    fn route_map_str<'a>(&self, direct: bool) -> &'a str;
    /// returns a url string used to submit a swap request to the api
    fn swap_str<'a>(&self) -> &'a str;
    /// returns a url string used to submit a quote request to the api
    fn quote_str(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: Slippage,
        fee_bps: FeeBps,
    ) -> String;
    /// returns a url string used to submit a price request to the api
    fn price_str(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> String;
    /// submit a blocking quote request
    fn quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: Slippage,
        fee_bps: FeeBps,
    ) -> anyhow::Result<Response<Vec<Quote>>>;
    /// submit a non-blocking quote request
    async fn async_quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: Slippage,
        fee_bps: FeeBps,
    ) -> anyhow::Result<Response<Vec<Quote>>>;
    /// submit a blocking swap request
    fn swap(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> anyhow::Result<Swap>;
    /// submit a non blocking swap request
    async fn async_swap(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> anyhow::Result<Swap>;
    /// submit a blocking route map request
    fn route_map(&self, direct: bool) -> anyhow::Result<RouteMap>;
    /// submit a non-blocking route map request
    async fn async_route_map(&self, direct: bool) -> anyhow::Result<RouteMap>;
    /// submit a blocking price request
    fn price(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> anyhow::Result<Vec<Price>>;
    /// submit a non-blocking price request
    async fn async_price(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> anyhow::Result<Vec<Price>>;
}

#[async_trait]
impl JupAPI for API {
    fn route_map_str<'a>(&self, direct: bool) -> &'a str {
        match self {
            // for v1 -> v3  same url is used
            Self::V6 => {
                if direct {
                    const DIRECT: &str =
                        "https://quote-api.jup.ag/v6/indexed-route-map?onlyDirectRoutes=true";
                    DIRECT
                } else {
                    const NOT_DIRECT: &str =
                        "https://quote-api.jup.ag/v6/indexed-route-map?onlyDirectRoutes=false";
                    NOT_DIRECT
                }
            }
            _ => {
                if direct {
                    const DIRECT: &str =
                        "https://quote-api.jup.ag/v6/indexed-route-map?onlyDirectRoutes=true";
                    DIRECT
                } else {
                    const NOT_DIRECT: &str =
                        "https://quote-api.jup.ag/v6/indexed-route-map?onlyDirectRoutes=false";
                    NOT_DIRECT
                }
            }
        }
    }
    fn swap_str<'a>(&self) -> &'a str {
        match self {
            // for v1 -> v3  same url is used
            Self::V6 => {
                const SWAP: &str = "https://quote-api.jup.ag/v6/swap";
                SWAP
            }
            _ => {
                const SWAP: &str = "https://quote-api.jup.ag/v6/swap";
                SWAP
            }
        }
    }
    fn quote_str(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: Slippage,
        fees_bps: FeeBps,
    ) -> String {
        let quote = match self {
            _ => {
                format!(
                    "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&onlyDirectRoutes={}&{}{}",
                    input_mint,
                    output_mint,
                    amount,
                    only_direct_routes,
                    slippage.value(),
                    fees_bps.value(),
                )
            }
            /*_ => {
                format!(
                    "https://quote-api.jup.ag/v1/quote?inputMint={}&outputMint={}&amount={}&onlyDirectRoutes={}&{}{}",
                    input_mint,
                    output_mint,
                    amount,
                    only_direct_routes,
                    slippage.value(),
                    fees_bps.value(),
                )
            }*/
        };
        log::info!("quote_str{quote}");
        quote
    }

    fn price_str(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> String {
        let formatted = format!("{:?}", input_mints);
        let formatted = formatted.replace('[', "");
        let formatted = formatted.replace(']', "");
        let formatted = formatted.replace(" ", "");
        match self {
            Self::V6 | Self::V4 => {
                format!(
                    "https://price.jup.ag/v4/price?ids={}&vsToken={}{}",
                    formatted,
                    output_mint,
                    if let Some(ui_amount) = ui_amount {
                        format!("&amount={}", ui_amount)
                    } else {
                        "".to_string()
                    },
                )
            }
            _ => {
                format!(
                    "https://quote-api.jup.ag/v1/price?id={}&vsToken={}{}",
                    formatted,
                    output_mint,
                    if let Some(ui_amount) = ui_amount {
                        format!("&amount={}", ui_amount)
                    } else {
                        "".to_string()
                    },
                )
            }
        }
    }
    fn swap(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> anyhow::Result<Swap> {
        let (url, request) = self.process_swap_input(route, user_public_key, swap_config);
        let response = maybe_jupiter_api_error::<crate::SwapResponse>(
            self.client()
                .post(url)
                .json(&request)
                .send()?
                .error_for_status()?
                .json()?,
        )?;

        self.process_swap_response(response)
    }
    async fn async_swap(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> anyhow::Result<Swap> {
        let (url, request) = self.process_swap_input(route, user_public_key, swap_config);
        let response = maybe_jupiter_api_error::<crate::SwapResponse>(
            self.async_client()
                .post(url)
                .json(&request)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?,
        )?;

        self.process_swap_response(response)
    }
    fn route_map(&self, direct: bool) -> anyhow::Result<RouteMap> {
        let url = self.route_map_str(direct);
        let response = self.client().get(url).send()?.json::<IndexedRouteMap>()?;
        self.process_route_map_response(response)
    }
    async fn async_route_map(&self, direct: bool) -> anyhow::Result<RouteMap> {
        let url = self.route_map_str(direct);
        let response = reqwest::get(url).await?.json::<IndexedRouteMap>().await?;
        self.process_route_map_response(response)
    }
    fn quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: Slippage,
        fees_bps: FeeBps,
    ) -> anyhow::Result<Response<Vec<Quote>>> {
        let url = self.quote_str(
            input_mint,
            output_mint,
            amount,
            only_direct_routes,
            slippage,
            fees_bps,
        );
        let response = self.client().get(url).send()?;
        let response_str = response.text()?;
        if let Ok(res) = serde_json::from_str(&response_str) {
            Ok(res)
        } else {
            Err(anyhow!("failed to deserialize response {}", response_str))
        }
    }
    async fn async_quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: Slippage,
        fees_bps: FeeBps,
    ) -> anyhow::Result<Response<Vec<Quote>>> {
        let url = self.quote_str(
            input_mint,
            output_mint,
            amount,
            only_direct_routes,
            slippage,
            fees_bps,
        );
        Ok(maybe_jupiter_api_error(
            self.async_client().get(url).send().await?.json().await?,
        )?)
    }
    fn price(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> anyhow::Result<Vec<Price>> {
        let url = self.price_str(input_mints, output_mint, ui_amount);
        if self.eq(&Self::V6) || self.eq(&Self::V4) {
            let res: crate::Response<HashMap<String, Price>> =
                maybe_jupiter_api_error(self.client().get(url).send()?.json()?)?;
            Ok(res.data.into_values().collect::<Vec<_>>())
        } else {
            let res: crate::Response<Vec<Price>> =
                maybe_jupiter_api_error(self.client().get(url).send()?.json()?)?;
            Ok(res.data)
        }
    }
    async fn async_price(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> anyhow::Result<Vec<Price>> {
        let url = self.price_str(input_mints, output_mint, ui_amount);
        if self.eq(&Self::V6) || self.eq(&Self::V4) {
            let res: crate::Response<HashMap<String, Price>> =
                maybe_jupiter_api_error(self.async_client().get(url).send().await?.json().await?)?;
            Ok(res.data.into_values().collect::<Vec<_>>())
        } else {
            let res: crate::Response<Vec<Price>> =
                maybe_jupiter_api_error(self.async_client().get(url).send().await?.json().await?)?;
            Ok(res.data)
        }
    }
}

impl API {
    pub fn process_swap_input<'a>(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> (&'a str, crate::SwapRequest) {
        let url = self.swap_str();

        let request = crate::SwapRequest {
            route,
            wrap_unwrap_SOL: swap_config.wrap_unwrap_sol,
            fee_account: swap_config.fee_account.map(|x| x.to_string()),
            token_ledger: swap_config.token_ledger.map(|x| x.to_string()),
            user_public_key,
        };
        (url, request)
    }

    pub fn process_swap_response(&self, response: SwapResponse) -> anyhow::Result<Swap> {
        fn decode(base64_transaction: String) -> anyhow::Result<crate::Transaction> {
            bincode::deserialize(&base64::decode(base64_transaction)?).map_err(|err| err.into())
        }

        Ok(Swap {
            setup: response.setup_transaction.map(decode).transpose()?,
            swap: decode(response.swap_transaction)?,
            cleanup: response.cleanup_transaction.map(decode).transpose()?,
        })
    }
    pub fn process_route_map_response(
        &self,
        response: IndexedRouteMap,
    ) -> anyhow::Result<RouteMap> {
        let mint_keys = response
            .mint_keys
            .into_iter()
            .map(|x| x.parse::<Pubkey>().map_err(|err| err.into()))
            .collect::<anyhow::Result<Vec<Pubkey>>>()?;

        let mut route_map = std::collections::HashMap::with_capacity(mint_keys.len());
        for (from_index, to_indices) in response.indexed_route_map {
            route_map.insert(
                mint_keys[from_index],
                to_indices.into_iter().map(|i| mint_keys[i]).collect(),
            );
        }
        Ok(route_map)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::JupAPI;
    use super::*;
    #[tokio::test]
    async fn test_jupapi_v6() {
        let got = API::V6.route_map_str(true);
        assert_eq!(
            got,
            "https://quote-api.jup.ag/v6/indexed-route-map?onlyDirectRoutes=true"
        );
        let got = API::V1.route_map_str(true);
        assert_eq!(
            got,
            "https://quote-api.jup.ag/v1/indexed-route-map?onlyDirectRoutes=true"
        );
        let got = API::V6.route_map_str(false);
        assert_eq!(
            got,
            "https://quote-api.jup.ag/v6/indexed-route-map?onlyDirectRoutes=false"
        );
        let got = API::V1.route_map_str(false);
        assert_eq!(
            got,
            "https://quote-api.jup.ag/v1/indexed-route-map?onlyDirectRoutes=false"
        );
        let got = API::V6.swap_str();
        assert_eq!(got, "https://quote-api.jup.ag/v6/swap");
        let got = API::V1.swap_str();
        assert_eq!(got, "https://quote-api.jup.ag/v1/swap");
        let price = API::V6.price_str(
            &[
                Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            ],
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            None,
        );
        assert_eq!(price, "https://price.jup.ag/v4/price?ids=So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&vsToken=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        let price = API::V4.price_str(
            &[
                Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            ],
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
            None,
        );
        assert_eq!(price, "https://price.jup.ag/v4/price?ids=So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&vsToken=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    }
}
