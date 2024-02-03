//! helper utility for quoting assets

use std::cmp::Ordering;

use anyhow::anyhow;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::ui_amount_to_amount;

use crate::{
    api::API,
    slippage::{FeeBps, Slippage},
    types::Quote,
};

#[derive(Clone, Copy)]
pub struct Quoter {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_mint_decimals: u8,
    pub output_mint_decimals: u8,
}

impl Quoter {
    pub fn new(
        rpc: &RpcClient,
        input_mint: Pubkey,
        output_mint: Pubkey,
    ) -> crate::types::Result<Self> {
        let input_mint_decimals = match rpc.get_account_data(&input_mint) {
            Ok(data) => match spl_token::state::Mint::unpack_unchecked(&data[..]) {
                Ok(dec) => dec.decimals,
                Err(err) => return Err(crate::error::Error::JupiterApi(err.to_string())),
            },
            Err(err) => return Err(crate::error::Error::JupiterApi(err.to_string())),
        };
        let output_mint_decimals = match rpc.get_account_data(&output_mint) {
            Ok(data) => match spl_token::state::Mint::unpack_unchecked(&data[..]) {
                Ok(dec) => dec.decimals,
                Err(err) => return Err(crate::error::Error::JupiterApi(err.to_string())),
            },
            Err(err) => return Err(crate::error::Error::JupiterApi(err.to_string())),
        };
        Ok(Self {
            input_mint,
            output_mint,
            input_mint_decimals,
            output_mint_decimals,
        })
    }
    /// Executes the configured search, returning
    /// a vector of quotes, sorted in descending order of output
    /// amount.
    ///
    /// To take the top three routes, you would take the first three
    /// elements of the returned vector.
    pub async fn lookup_routes(
        self,
        ui_amount: f64,
        direct: bool,
        slippage: Slippage,
        fees_bps: FeeBps,
        version: API,
    ) -> anyhow::Result<Vec<Quote>> {
        let mut quotes = crate::AsyncClient
            .quote(
                self.input_mint,
                self.output_mint,
                ui_amount_to_amount(ui_amount, self.input_mint_decimals),
                direct,
                slippage,
                fees_bps,
                version,
            )
            .await?;
        Ok(quotes.data)
    }
    pub fn lookup_routes2(
        self,
        ui_amount: f64,
        direct: bool,
        slippage: Slippage,
        fees_bps: FeeBps,
        version: API,
    ) -> anyhow::Result<Vec<Quote>> {
        let mut quotes = match crate::Client.quote(
            self.input_mint,
            self.output_mint,
            ui_amount_to_amount(ui_amount, self.input_mint_decimals),
            direct,
            slippage,
            fees_bps,
            version,
        ) {
            Ok(quotes) => quotes,
            Err(err) => return Err(anyhow!("failed to lookup quote {:#?}", err)),
        };

        Ok(quotes.data)
    }
}