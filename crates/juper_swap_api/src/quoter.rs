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
        quotes.data.sort_unstable_by(cmp_quote);

        if !quotes.data.is_empty() {
            let first_quote = &quotes.data[0];
            let last_quote = &quotes.data[quotes.data.len() - 1];
            log::debug!(
                "first_quote(in={}, out={}, out_slip={}), last_quote(in={}, out={}, out_slip={})",
                first_quote.in_amount,
                first_quote.out_amount,
                first_quote.out_amount_with_slippage,
                last_quote.in_amount,
                last_quote.out_amount,
                last_quote.out_amount_with_slippage
            )
        }
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

        quotes.data.sort_unstable_by(cmp_quote);
        if !quotes.data.is_empty() {
            let first_quote = &quotes.data[0];
            let last_quote = &quotes.data[quotes.data.len() - 1];
            log::debug!(
                "first_quote(in={}, out={}, out_slip={}), last_quote(in={}, out={}, out_slip={})",
                first_quote.in_amount,
                first_quote.out_amount,
                first_quote.out_amount_with_slippage,
                last_quote.in_amount,
                last_quote.out_amount,
                last_quote.out_amount_with_slippage
            )
        }
        Ok(quotes.data)
    }
}

/// intended to reverse sort the quotes, such as that
/// the first elements are greater in output amountr
pub fn cmp_quote(a: &Quote, b: &Quote) -> Ordering {
    if a.out_amount_with_slippage == 0 {
        return Ordering::Greater;
    }
    if b.out_amount_with_slippage == 0 {
        return Ordering::Greater;
    }
    if a.out_amount_with_slippage < b.out_amount_with_slippage {
        Ordering::Greater
    } else if a.out_amount_with_slippage > b.out_amount_with_slippage {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}
