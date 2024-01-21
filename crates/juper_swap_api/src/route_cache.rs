use std::collections::HashMap;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::{
    api::API,
    quoter::Quoter,
    slippage::{FeeBps, Slippage},
    types::Quote,
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct WrappedQuote {
    pub quote: Quote,
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
}

impl RouteCache {
    pub fn new(size: usize) -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::with_capacity(size))),
            quoters: Arc::new(RwLock::new(HashMap::with_capacity(size))),
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
        version: API,
    ) -> anyhow::Result<()> {
        for (input, output) in tokens.iter() {
            let quoter = if let Some(quoter) = self.quoters.read().unwrap().get(&(*input, *output))
            {
                *quoter
            } else {
                Quoter::new(rpc, *input, *output)?
            };
            let routes = quoter
                .lookup_routes(ui_amount, false, slippage, FeeBps::Zero, version)
                .await?;
            match self.routes.write() {
                Ok(mut entry_lock) => {
                    if let Some(cache_entry) = entry_lock.get_mut(&(*input, *output)) {
                        let counter_prev = cache_entry.counter;
                        cache_entry.counter += 1;
                        cache_entry.quotes.clear();
                        cache_entry.quotes.append(
                            &mut routes
                                .into_iter()
                                .map(|quote| WrappedQuote {
                                    quote,
                                    stale: false,
                                })
                                .collect::<Vec<_>>(),
                        );
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
                                quotes: routes
                                    .into_iter()
                                    .map(|quote| WrappedQuote {
                                        quote,
                                        stale: false,
                                    })
                                    .collect::<Vec<_>>(),
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
