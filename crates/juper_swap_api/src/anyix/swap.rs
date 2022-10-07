//! provides functions to create AnyIx jupiter swaps using the swap api
use crate::{
    slippage::{FeeBps, Slippage},
    types::{Quote, SwapConfig},
    utils::decompile_transaction_instructions,
};
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::InstructionData;
use anchor_lang::{
    prelude::AccountMeta, solana_program::instruction::InstructionError, ToAccountMetas,
};
use anyhow::{anyhow, Result};

use regex::RegexSet;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::Instruction, signature::Signature};
use solana_sdk::{message::SanitizedMessage, signer::Signer};

#[cfg(test)]
use solana_client::rpc_client::serialize_and_encode;

use std::{collections::HashMap, sync::Arc};

use super::{replace_accounts, replace_by_account_pubkey, MARKET_BLACKLIST};

#[derive(Clone, Default)]
pub struct JupiterAnyIxSwap {
    /// vector of instructions to use for the setup process, currently unused
    pub setup: Option<Vec<Instruction>>,
    /// the actual anyix swap instruction
    pub swap: Option<Instruction>,
    /// vector of instructions to use for the cleanup process, currently unused
    pub cleanup: Option<Vec<Instruction>>,
}

/// returns the encoded Instruction
pub fn new_anyix_swap_ix_with_quote(
    swap_route: Quote,
    rpc: &Arc<RpcClient>,
    payer: &dyn Signer,
    anyix_program: Pubkey,
    management: Pubkey,
    vault: Pubkey,
    vault_pda: Pubkey,
    // a map of keys to replace, and the values to replace them with
    replacements: &HashMap<Pubkey, Pubkey>,
    // if ture, and transaction setup is required failed
    // otherwise warn
    fail_on_setup: bool,
) -> Result<JupiterAnyIxSwap> {
    let jup_client = crate::Client::new();
    let swap_config = jup_client.swap_with_config(
        swap_route,
        vault_pda,
        SwapConfig {
            wrap_unwrap_sol: Some(false),
            ..Default::default()
        },
    )?;

    let crate::types::Swap {
        setup,
        mut swap,
        cleanup,
    } = swap_config;
    let mut jup_any_ix = JupiterAnyIxSwap::default();
    if setup.is_some() && !fail_on_setup {
        if let Some(setup) = setup {
            jup_any_ix.setup = Some(decompile_transaction_instructions(setup)?)
        }
    } else if setup.is_some() && fail_on_setup {
        return Err(anyhow!("abort on transaction setup enabled"));
    }

    jup_any_ix.swap = match process_transaction(
        rpc,
        payer,
        &mut swap,
        vault,
        anyix_program,
        management,
        replacements,
    ) {
        Ok(ix) => Some(ix),
        Err(err) => {
            let error_msg = format!("tx process failed {:#?}", err);
            log::debug!("{}", error_msg);
            return Err(anyhow!("{}", error_msg));
        }
    };
    if cleanup.is_some() {
        log::warn!("transaction cleanup not yet supported");
    }
    Ok(jup_any_ix)
}

/// creates, and sends an AnyIx jupiter swap using the given quote
pub fn new_anyix_swap_with_quote(
    swap_route: Quote,
    rpc: &Arc<RpcClient>,
    payer: &dyn Signer,
    anyix_program: Pubkey,
    management: Pubkey,
    vault: Pubkey,
    vault_pda: Pubkey,
    skip_preflight: bool,
    replacements: &HashMap<Pubkey, Pubkey>,
    fail_on_setup: bool,
) -> Result<Signature> {
    let jup_any_ix = new_anyix_swap_ix_with_quote(
        swap_route,
        rpc,
        payer,
        anyix_program,
        management,
        vault,
        vault_pda,
        replacements,
        fail_on_setup,
    )?;
    let jup_swap_ix = if let Some(swap_ix) = jup_any_ix.swap {
        swap_ix
    } else {
        return Err(anyhow!("failed to create jupiter any ix swap"));
    };
    let mut tx = Transaction::new_with_payer(&[jup_swap_ix], Some(&payer.pubkey()));

    #[cfg(test)]
    println!(
        "encoded jupiter tx {}",
        serialize_and_encode(
            &tx,
            solana_transaction_status::UiTransactionEncoding::Base64
        )
        .unwrap()
    );

    tx.sign(&vec![payer], rpc.get_latest_blockhash()?);
    log::debug!("sending jupiter swap ix");
    if skip_preflight {
        match rpc.send_transaction_with_config(
            &tx,
            RpcSendTransactionConfig {
                skip_preflight,
                ..Default::default()
            },
        ) {
            Ok(sig) => Ok(sig),
            Err(err) => {
                let error_msg = format!("failed to send jupiter swap ix {:#?}", err);
                log::debug!("{}", error_msg);
                return Err(anyhow!("{}", error_msg));
            }
        }
    } else {
        match rpc.send_and_confirm_transaction(&tx) {
            Ok(sig) => return Ok(sig),
            Err(err) => {
                let error_msg = format!("failed to send jupiter swap ix {:#?}", err);
                log::debug!("{}", error_msg);
                return Err(anyhow!("{}", error_msg));
            }
        }
    }
}

/// given an input and output mint, find up to `max_tries` routes
/// which will be executed sequentially until the first one succeeds
pub fn new_anyix_swap(
    rpc: &Arc<RpcClient>,
    payer: &dyn Signer,
    anyix_program: Pubkey,
    management: Pubkey,
    vault: Pubkey,
    vault_pda: Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    input_amount: f64,
    skip_preflight: bool,
    max_tries: usize,
    disallowed_market_list: Option<RegexSet>,
    replacements: &HashMap<Pubkey, Pubkey>,
    slippage: Slippage,
    fail_on_setup: bool,
) -> Result<Signature> {
    let market_blacklist: RegexSet = if let Some(list) = disallowed_market_list {
        list
    } else {
        MARKET_BLACKLIST.clone()
    };
    let quoter = crate::quoter::Quoter::new(rpc, input_mint, output_mint)?;
    let routes = quoter
        .lookup_routes2(input_amount, false, slippage, FeeBps::Zero)?
        .into_iter()
        .filter(|quote| {
            for market_info in quote.market_infos.iter() {
                if market_blacklist.is_match(&market_info.label) {
                    log::warn!("ignoring market {}", market_info.label);
                    return false;
                } else {
                    continue;
                }
            }
            true
        })
        .collect::<Vec<_>>();
    if routes.is_empty() {
        return Err(anyhow!("failed to find any routes"));
    }
    for route in routes.into_iter().take(max_tries) {
        let swap_fn = |swap_route: Quote| -> Result<Signature> {
            new_anyix_swap_with_quote(
                swap_route,
                rpc,
                payer,
                anyix_program,
                management,
                vault,
                vault_pda,
                skip_preflight,
                replacements,
                fail_on_setup,
            )
        };
        if let Ok(sig) = swap_fn(route.clone()) {
            return Ok(sig);
        }
    }
    Err(anyhow!("failed to process jupiter swap"))
}

/// processes a transaction as returned from the jupiter swap api, into a format suitable for
/// execution with AnyIx
pub fn process_transaction(
    rpc: &Arc<RpcClient>,
    payer: &dyn Signer,
    tx: &mut Transaction,
    vault: Pubkey,
    anyix_program: Pubkey,
    management: Pubkey,
    replacements: &HashMap<Pubkey, Pubkey>,
) -> Result<Instruction> {
    let mut instructions = decompile_transaction_instructions(tx.clone())?;
    // ensure all instructions invoke a program_id that is whitelisted
    for ix in instructions.iter() {
        // prevent a rogue api from returning programs that are not the ata or jupiter progarm
        if ix.program_id.ne(&spl_associated_token_account::ID)
            && ix.program_id.ne(&juper_swap_cpi::JUPITER_V3_AGG_ID)
        {
            return Err(anyhow!("invalid program id {}", ix.program_id));
        }
    }
    let any_ix_args = instructions
        .iter_mut()
        .filter_map(|ix| {
            // first set any signer accounts as non signers
            // this is to prevent txn signing issues where the swap ix requires a pda to sign
            //
            // however if the instruction is for the ATA program we dont do this
            if ix.program_id.eq(&juper_swap_cpi::JUPITER_V3_AGG_ID) {
                ix.accounts = ix
                    .accounts
                    .iter_mut()
                    .map(|account| {
                        account.is_signer = false;
                        account.clone()
                    })
                    .collect();
                let (jup_ix, swap_input) =
                    match juper_swap_cpi::process_jupiter_instruction(&ix.data[..]) {
                        Ok(ix) => ix,
                        Err(err) => {
                            log::error!("failed to process jupiter ix {:#?}", err);
                            return None;
                        }
                    };
                if let Ok(args) = super::new_jupiter_swap_ix_data(ix.clone(), jup_ix, swap_input) {
                    Some(args)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut accounts = juper_swap_cpi::accounts::JupiterSwap {
        vault,
        authority: payer.pubkey(),
        jupiter_program: juper_swap_cpi::JUPITER_V3_AGG_ID,
        management,
    }
    .to_account_metas(Some(true));
    let any_ix = ::anyix::AnyIx {
        num_instructions: any_ix_args.len() as u8,
        instruction_data_sizes: any_ix_args.iter().map(|ix| ix.data.len() as u8).collect(),
        instruction_account_counts: any_ix_args
            .iter()
            .map(|ix| ix.accounts.len() as u8)
            .collect(),
        instruction_datas: any_ix_args.iter().map(|ix| ix.data.clone()).collect(),
    }
    .pack()?;
    accounts.extend_from_slice(
        &any_ix_args
            .iter()
            .flat_map(|ix| ix.accounts.clone())
            .collect::<Vec<_>>()[..],
    );
    let mut ix = Instruction {
        program_id: anyix_program,
        accounts,
        data: juper_swap_cpi::instructions::JupiterSwapArgs { input_data: any_ix }.data(),
    };
    replace_accounts(&mut ix, rpc, &mut replace_by_account_pubkey, replacements)?;
    Ok(ix)
}
