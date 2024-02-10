use std::collections::HashMap;

use anyhow::anyhow;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};

use crate::{quote_types::{QuoteResponse, RequestOption}, slippage::Slippage};
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
pub struct Quoter {
 pub input_mint: Pubkey,
 pub output_mint: Pubkey,
 pub input_mint_decimals: u8,
 pub output_mint_decimals: u8,
}


#[derive(Clone)]
pub struct WrappedQuote {
    pub quote: QuoteResponse,
    pub stale: bool,
}

#[derive(Clone)]
pub struct RouteCacheEntry {
    pub counter: u128,
    pub quotes: Vec<WrappedQuote>,
}

//pub type RouteCache = Arc<RwLock<HashMap<(Pubkey, Pubkey), RouteCacheEntry>>>;

#[derive(Clone)]
pub struct RouteCache {
    pub routes: Arc<RwLock<HashMap<(Pubkey, Pubkey), RouteCacheEntry>>>,
    pub quoters: Arc<RwLock<HashMap<(Pubkey, Pubkey), Quoter>>>,
    c: Arc<crate::Client>
}

impl RouteCache {
    pub fn new(size: usize) -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::with_capacity(size))),
            quoters: Arc::new(RwLock::new(HashMap::with_capacity(size))),
            c: Arc::new(crate::Client::new().unwrap())
        }
    }
    pub fn mark_routes_stale(
        &self,
        tokens: &[(Pubkey /*input */, Pubkey /*output */)],
    ) -> anyhow::Result<()> {
        match self.routes.write() {
            Ok(mut cache_lock) => {
                tokens.iter().for_each(|token| {
                    if let Some(route) = cache_lock.get_mut(token) {
                        route.quotes.iter_mut().for_each(|quote| quote.stale = true);
                    }
                });
                Ok(())
            }
            Err(err) => Err(anyhow::anyhow!("failed to get route lock {:#?}", err)),
        }
    }
    pub async fn populate(
        &self,
        rpc: &RpcClient,
        tokens: &[(Pubkey /*input */, Pubkey /*output */)],
        slippage: Slippage,
        ui_amount: f64,
    ) -> anyhow::Result<()> {
        for (input, output) in tokens.iter() {
            let quoter = if let Some(quoter) = self.quoters.read().unwrap().get(&(*input, *output))
            {
                *quoter
            } else {
                Quoter::new(rpc, *input, *output)?
            };
            let routes = self.c.new_quote(
                &quoter.input_mint.to_string(),
                &quoter.output_mint.to_string(),
                spl_token::ui_amount_to_amount(ui_amount, quoter.input_mint_decimals),
                &[RequestOption::AsLegacyTransaction]
            )?;
            match self.routes.write() {
                Ok(mut entry_lock) => {
                    if let Some(cache_entry) = entry_lock.get_mut(&(*input, *output)) {
                        let counter_prev = cache_entry.counter;
                        cache_entry.counter += 1;
                        cache_entry.quotes.clear();
                        cache_entry.quotes.push(WrappedQuote {
                            quote: routes,
                            stale: false
                        });
                        log::info!(
                            "route_cache_update(old_counter={}, new_counter={})",
                            counter_prev,
                            cache_entry.counter
                        );
                    } else {
                        entry_lock.insert(
                            (*input, *output),
                            RouteCacheEntry {
                                counter: 0,
                                quotes: vec![WrappedQuote {
                                    quote: routes,
                                    stale: false
                                }],
                            },
                        );
                    }
                }
                Err(err) => {
                    log::error!("failed to get route cache entry lock {:#?}", err);
                }
            }
        }

        Ok(())
    }

    /// return value is (cache_counter_updates, quote_)
    pub fn top_n_routes(
        &self,
        input: Pubkey,
        output: Pubkey,
        n: usize,
    ) -> anyhow::Result<Option<(u128, Vec<WrappedQuote>)>> {
        match self.routes.read() {
            Ok(cache_lock) => {
                if let Some(value) = cache_lock.get(&(input, output)) {
                    return Ok(Some((
                        value.counter,
                        value.quotes.iter().take(n).cloned().collect::<Vec<_>>(),
                    )));
                } else {
                    log::warn!("found no routes for input {} output {}", input, output);
                    Ok(None)
                }
            }
            Err(err) => Err(anyhow::anyhow!("failed to lock cache {:#?}", err)),
        }
    }
}


impl Quoter {
    pub fn new(
        rpc: &RpcClient,
        input_mint: Pubkey,
        output_mint: Pubkey,
    ) -> anyhow::Result<Self> {
        let input_mint_decimals = match rpc.get_account_data(&input_mint) {
            Ok(data) => match spl_token::state::Mint::unpack_unchecked(&data[..]) {
                Ok(dec) => dec.decimals,
                Err(err) => return Err(anyhow!("failed to unpack input mint {err:#?}")),
            },
            Err(err) => return Err(anyhow!("failed to fetch input mint account")),
        };
        let output_mint_decimals = match rpc.get_account_data(&output_mint) {
            Ok(data) => match spl_token::state::Mint::unpack_unchecked(&data[..]) {
                Ok(dec) => dec.decimals,
                Err(err) => return Err(anyhow!("failed to unpack output mint {err:#?}")),
            },
            Err(err) => return Err(anyhow!("failed to fetch output mint account")),
        };
        Ok(Self {
            input_mint,
            output_mint,
            input_mint_decimals,
            output_mint_decimals,
        })
    }
}