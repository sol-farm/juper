use crate::api::JupAPI;

use solana_sdk::{pubkey::Pubkey, transaction::Transaction};

//pub mod core;
pub mod anyix;
pub mod api;
pub mod error;
mod field_as_string;
pub mod quoter;
pub mod route_cache;
pub mod slippage;
pub mod types;
pub mod utils;

use types::{Price, Quote, Response, RouteMap, Swap, SwapConfig, SwapRequest, SwapResponse};

#[derive(Clone, Copy)]
/// Implements a blocking client for the Jupiter Aggregator.
pub struct Client;

#[derive(Clone, Copy)]
/// Implements a non blocking client for the Jupiter aggregator
pub struct AsyncClient;

impl Client {
    pub fn new() -> Self {
        Self
    }
    /// Get swap serialized transactions for a quote
    pub fn swap_with_config(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> anyhow::Result<Swap> {
        crate::api::API::V1.swap(route, user_public_key, swap_config)
    }
    /// Get swap serialized transactions for a quote using `SwapConfig` defaults
    pub fn swap(&self, route: Quote, user_public_key: Pubkey) -> anyhow::Result<Swap> {
        self.swap_with_config(route, user_public_key, SwapConfig::default())
    }
    /// Returns a hash map, input mint as key and an array of valid output mint as values
    pub fn route_map(&self, only_direct_routes: bool) -> anyhow::Result<RouteMap> {
        crate::api::API::V1.route_map(only_direct_routes)
    }
    /// Get quote for a given input mint, output mint and amount
    pub fn quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: crate::slippage::Slippage,
        fees_bps: crate::slippage::FeeBps,
    ) -> anyhow::Result<Response<Vec<Quote>>> {
        crate::api::API::V1.quote(
            input_mint,
            output_mint,
            amount,
            only_direct_routes,
            slippage,
            fees_bps,
        )
    }
    /// Get simple price for a given input mint, output mint and amount
    pub fn price(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
        v4: bool,
    ) -> anyhow::Result<Vec<Price>> {
        if v4 {
            crate::api::API::V4.price(input_mints, output_mint, ui_amount)
        } else {
            crate::api::API::V1.price(input_mints, output_mint, ui_amount)
        }
    }
    pub fn batch_price_lookup(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
        v4: bool,
    ) -> anyhow::Result<Vec<Price>> {
        let input_len = input_mints.len();
        if input_len <= 10 {
            if v4 {
                return crate::api::API::V4.price(input_mints, output_mint, ui_amount);
            } else {
                return crate::api::API::V1.price(input_mints, output_mint, ui_amount);
            }
        } else {
            let chunks = input_mints.chunks(10);
            let mut prices = Vec::with_capacity(input_len);
            chunks.into_iter().for_each(|chunk| {
                if v4 {
                    if let Ok(price_infos) =
                        crate::api::API::V4.price(chunk, output_mint, ui_amount)
                    {
                        prices.extend_from_slice(&price_infos[..]);
                    }
                } else {
                    if let Ok(price_infos) =
                        crate::api::API::V1.price(chunk, output_mint, ui_amount)
                    {
                        prices.extend_from_slice(&price_infos[..]);
                    }
                }
            });
            Ok(prices)
        }
    }
    pub async fn async_batch_price_lookup(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
        v4: bool,
    ) -> anyhow::Result<Vec<Price>> {
        let input_len = input_mints.len();
        if input_len <= 10 {
            if v4 {
                return Ok(crate::api::API::V4
                    .async_price(input_mints, output_mint, ui_amount)
                    .await?);
            } else {
                return Ok(crate::api::API::V1
                    .async_price(input_mints, output_mint, ui_amount)
                    .await?);
            }
        } else {
            let chunks = input_mints.chunks(10);
            let mut prices = Vec::with_capacity(input_len);
            for chunk in chunks {
                if v4 {
                    if let Ok(price_infos) = crate::api::API::V4
                        .async_price(chunk, output_mint, ui_amount)
                        .await
                    {
                        prices.extend_from_slice(&price_infos[..]);
                    }
                } else {
                    if let Ok(price_infos) = crate::api::API::V1
                        .async_price(chunk, output_mint, ui_amount)
                        .await
                    {
                        prices.extend_from_slice(&price_infos[..]);
                    }
                }
            }
            Ok(prices)
        }
    }
}

impl AsyncClient {
    pub fn new() -> Self {
        Self
    }
    /// Get swap serialized transactions for a quote
    pub async fn swap_with_config(
        &self,
        route: Quote,
        user_public_key: Pubkey,
        swap_config: SwapConfig,
    ) -> anyhow::Result<Swap> {
        crate::api::API::V1
            .async_swap(route, user_public_key, swap_config)
            .await
    }
    /// Get swap serialized transactions for a quote using `SwapConfig` defaults
    pub async fn swap(&self, route: Quote, user_public_key: Pubkey) -> anyhow::Result<Swap> {
        let conf = SwapConfig {
            wrap_unwrap_sol: Some(false),
            ..Default::default()
        };
        self.swap_with_config(route, user_public_key, conf).await
    }
    /// Returns a hash map, input mint as key and an array of valid output mint as values
    pub async fn route_map(&self, only_direct_routes: bool) -> anyhow::Result<RouteMap> {
        crate::api::API::V1
            .async_route_map(only_direct_routes)
            .await
    }
    /// Get quote for a given input mint, output mint and amount
    pub async fn quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        only_direct_routes: bool,
        slippage: crate::slippage::Slippage,
        fees_bps: crate::slippage::FeeBps,
    ) -> anyhow::Result<Response<Vec<Quote>>> {
        crate::api::API::V1
            .async_quote(
                input_mint,
                output_mint,
                amount,
                only_direct_routes,
                slippage,
                fees_bps,
            )
            .await
    }
    /// Get simple price for a given input mint, output mint and amount
    pub async fn price(
        &self,
        input_mints: &[Pubkey],
        output_mint: Pubkey,
        ui_amount: Option<f64>,
        v4: bool,
    ) -> anyhow::Result<Vec<Price>> {
        if v4 {
            crate::api::API::V4
                .async_price(input_mints, output_mint, ui_amount)
                .await
        } else {
            crate::api::API::V1
                .async_price(input_mints, output_mint, ui_amount)
                .await
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AsyncClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::JupAPI;
    use super::*;
    #[test]
    fn test_jupapi_v1() {
        let prices = Client::new()
            .price(
                &[
                    Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                    Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                ],
                Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                None,
                false,
            )
            .unwrap();
        assert!(prices.len() == 2);
        println!("{:#?}", prices);

        let prices = Client::new()
            .price(
                &[
                    Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                    Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                ],
                Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                None,
                true,
            )
            .unwrap();
        assert!(prices.len() == 2);
        println!("{:#?}", prices);
    }
    #[tokio::test]
    async fn test_jupapi_v1_async() {
        let prices = AsyncClient::new()
            .price(
                &[
                    Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                    Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                ],
                Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                None,
                false,
            )
            .await
            .unwrap();
        assert!(prices.len() == 2);
        println!("{:#?}", prices);
        let prices = AsyncClient::new()
            .price(
                &[
                    Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
                    Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                ],
                Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
                None,
                true,
            )
            .await
            .unwrap();
        assert!(prices.len() == 2);
        println!("{:#?}", prices);
    }
}
