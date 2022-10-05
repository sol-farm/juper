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
        input_mint: Pubkey,
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> anyhow::Result<Response<Price>> {
        crate::api::API::V1.price(input_mint, output_mint, ui_amount)
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
        input_mint: Pubkey,
        output_mint: Pubkey,
        ui_amount: Option<f64>,
    ) -> anyhow::Result<Response<Price>> {
        crate::api::API::V1
            .async_price(input_mint, output_mint, ui_amount)
            .await
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
