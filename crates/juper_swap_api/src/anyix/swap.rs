//! provides functions to create AnyIx jupiter swaps using the swap api
use crate::{
    quote_types::{QuoteResponse, RequestOption}, slippage::{FeeBps, Slippage}, utils::decompile_transaction_instructions
};
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anyhow::{anyhow, Result};

use juper_swap_cpi::JupiterIx;
use regex::RegexSet;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::Instruction, signature::Signature};
use solana_sdk::{program_pack::Pack, signer::Signer};

#[cfg(test)]
use solana_client::rpc_client::serialize_and_encode;

use std::{collections::HashMap, sync::Arc};

use super::{replace_accounts, replace_by_account_pubkey, MARKET_BLACKLIST};

/// Wraps the setup, swap, and cleanup transactions decoded from
/// jupiters swap api.
///
/// This is slightly modified from the original response, as it does not contain transactions
/// rather the individual instruction(s) to be used, as execution of these instructions
/// is being delegated to AnyIx
#[derive(Clone, Default)]
pub struct JupiterAnyIxSwap {
    /// vector of instructions to use for the setup process, currently unused
    pub setup: Option<Vec<Instruction>>,
    /// the actual anyix swap instruction
    pub swap: Option<Instruction>,
    /// vector of instructions to use for the cleanup process, currently unused
    pub cleanup: Option<Vec<Instruction>>,
}

/// given a specific trade route `swap_route`, request a transaction
/// from jupiter's swap api, decode the included transactions into
/// their individual instructions, and then encode the swap related
/// instructions into the AnyIx instruction format
pub fn new_anyix_swap_ix_with_quote(
    swap_route: QuoteResponse,
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
    input_mint: Pubkey,
    output_mint: Pubkey,
) -> Result<JupiterAnyIxSwap> {
    let jup_client = crate::Client::new()?;
    let swap_response = jup_client.new_swap(
        swap_route,
        &vault_pda.to_string(),
        false
    )?;
    let mut jup_any_ix = JupiterAnyIxSwap::default();
    if !swap_response.setup_instructions.is_empty() {
        jup_any_ix.setup = Some(swap_response.setup_instructions.iter().filter_map(|ix| ix.to_instruction().ok()).collect::<Vec<_>>())
    }
    let mut tx = swap_response.new_transaction(rpc, payer.pubkey(), None, None, input_mint)?;
    tx.sign(&vec![payer], rpc.get_latest_blockhash()?);
    jup_any_ix.swap = match process_transaction(
        rpc,
        payer,
        &mut tx,
        vault,
        anyix_program,
        management,
        replacements,
        input_mint,
        output_mint,
    ) {
        Ok(ix) => Some(ix),
        Err(err) => {
            let error_msg = format!("tx process failed {:#?}", err);
            log::debug!("{}", error_msg);
            return Err(anyhow!("{}", error_msg));
        }
    };
    Ok(jup_any_ix)
}

/// given a specific trade route `swap_route`, parse and execute
/// the trade via AnyIX
///
///  `anyix_program` will be the program implementing the `jupiter_swap` function
/// that decodes AnyIx instruction data.
///
/// The `disallowed_market_list` regex set contains a list of markets
/// that will cause any containing routes to be filtered out from the list
/// of available routes. If set to `None` the default blacklist is used.
///
/// The `replacements` map is used to replace accounts returned by
/// jupiters swap api. The keys of the map are the accounts to replace
/// while the values are the accounts to replace them with
///
/// When `fail_on_setup` is true, if the transaction returned via
/// jupiter's swap api requires setup, an error is returned, advancing
/// the `max_tries` count by 1.
pub fn new_anyix_swap_with_quote(
    swap_route: crate::quote_types::QuoteResponse,
    rpc: &Arc<RpcClient>,
    payer: &dyn Signer,
    anyix_program: Pubkey,
    management: Pubkey,
    vault: Pubkey,
    vault_pda: Pubkey,
    skip_preflight: bool,
    replacements: &HashMap<Pubkey, Pubkey>,
    fail_on_setup: bool,
    input_mint: Pubkey,
    output_mint: Pubkey,
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
        input_mint,
        output_mint,
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
                Err(anyhow!("{}", error_msg))
            }
        }
    } else {
        match rpc.send_and_confirm_transaction(&tx) {
            Ok(sig) => Ok(sig),
            Err(err) => {
                let error_msg = format!("failed to send jupiter swap ix {:#?}", err);
                log::debug!("{}", error_msg);
                Err(anyhow!("{}", error_msg))
            }
        }
    }
}

/// Given a specific input and output mint, find up to `max_tries`
/// routes that will be swapped through sequentially, stopping
/// on the first success or once `max_tries`is reached.
///
/// `anyix_program` will be the program implementing the `jupiter_swap` function
/// that decodes AnyIx instruction data.
///
/// The `disallowed_market_list` regex set contains a list of markets
/// that will cause any containing routes to be filtered out from the list
/// of available routes. If set to `None` the default blacklist is used.
///
/// The `replacements` map is used to replace accounts returned by
/// jupiters swap api. The keys of the map are the accounts to replace
/// while the values are the accounts to replace them with
///
/// When `fail_on_setup` is true, if the transaction returned via
/// jupiter's swap api requires setup, an error is returned, advancing
/// the `max_tries` count by 1.
pub fn new_anyix_swap(
    client: Arc<crate::Client>,
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
    let quoter = crate::route_cache::Quoter::new(rpc, input_mint, output_mint)?;
    let route = client.new_quote(
        &input_mint.to_string(),
        &output_mint.to_string(),
        spl_token::ui_amount_to_amount(input_amount, quoter.input_mint_decimals),
        &[RequestOption::AsLegacyTransaction]
    )?;
    match new_anyix_swap_with_quote(
        route,
        rpc,
        payer,
        anyix_program,
        management,
        vault,
        vault_pda,
        skip_preflight,
        replacements,
        fail_on_setup,
        input_mint,
        output_mint,
    ) {
        Ok(sig) => return Ok(sig),
        Err(err) => {
            return Err(anyhow!("anyix swap failed {:#?}", err));
        }
    }
}

/// Processes a decoded transaction returned from jupiter's swap api. This
/// should only be used with the swap transaction however it can be used
/// with the setup and cleanup transactions but that is not officially supported.
///
/// The transaction is decompiled into it's individual instructions, which are
/// then encoded into the AnyIX format, and a single instruction is returned
/// that can be used to execute the swap transaction proxied through any program
/// that implements the `jupiter_swap` AnyIx instruction.
pub fn process_transaction(
    rpc: &Arc<RpcClient>,
    payer: &dyn Signer,
    tx: &mut Transaction,
    vault: Pubkey,
    anyix_program: Pubkey,
    management: Pubkey,
    replacements: &HashMap<Pubkey, Pubkey>,
    input_mint: Pubkey,
    output_mint: Pubkey,
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
    // after filtering out the ata instructions, we need to make sure
    // that we have the correct number of instructions still
    let mut expected_instructions = 0;
    let any_ix_args = instructions
        .iter_mut()
        .filter_map(|ix| {
            // first set any signer accounts as non signers
            // this is to prevent txn signing issues where the swap ix requires a pda to sign
            //
            // however if the instruction is for the ATA program we dont do this
            if ix.program_id.eq(&juper_swap_cpi::JUPITER_V3_AGG_ID) {
                // increase the expected instructions before attempting decompilation
                // this will help us catch any swaps that request invalid routes
                expected_instructions += 1;
                ix.accounts = ix
                    .accounts
                    .iter_mut()
                    .map(|account| {
                        account.is_signer = false;
                        account.clone()
                    })
                    .collect();
                let (jup_ix, mut swap_input) =
                    match juper_swap_cpi::decode_jupiter_instruction(&ix.data[..]) {
                        Ok(ix) => ix,
                        Err(err) => {
                            log::error!("failed to process jupiter ix {:#?}", err);
                            return None;
                        }
                    };
                // for whirlpool swaps we need to manually specify the direction of the swap
                if jup_ix == JupiterIx::Whirlpool {
                    let token_vault_a = ix.accounts[5].pubkey;
                    match rpc.get_account_data(&token_vault_a) {
                        Ok(data) => match spl_token::state::Account::unpack_unchecked(&data[..]) {
                            Ok(token_vault_a_account) => {
                                if token_vault_a_account.mint.eq(&input_mint) {
                                    log::debug!("whirlpool swap, setting side to 0 (ask)");
                                    swap_input.side = 0;
                                } else if token_vault_a_account.mint.eq(&output_mint) {
                                    log::debug!("whirlpool swap, setting side to 1 (bid)");
                                    swap_input.side = 1;
                                }
                            }
                            Err(err) => {
                                log::error!("failed to process jupiter ix {:#?}", err);
                                return None;
                            }
                        },
                        Err(err) => {
                            log::error!("failed to process jupiter ix {:#?}", err);
                            return None;
                        }
                    }
                }
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
    if expected_instructions.ne(&any_ix_args.len()) {
        return Err(anyhow!(
            "unexpected instruction count. got {}, want {}",
            any_ix_args.len(),
            expected_instructions
        ));
    }
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
