//! provides integration of anyix and v2 vaults
use crate::{
    slippage::{FeeBps, Slippage},
    types::SwapConfig,
};
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::InstructionData;
use anchor_lang::{
    prelude::AccountMeta, solana_program::instruction::InstructionError, Accounts, ToAccountMetas,
};
use anyhow::{anyhow, Result};
use juper_swap_cpi::{JupiterIx, SwapInputs};
use solana_client::rpc_client::{serialize_and_encode, RpcClient};
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::{commitment_config::CommitmentConfig, message::SanitizedMessage, signer::Signer};
use solana_transaction_status::UiTransactionEncoding;
use static_pubkey::static_pubkey;
use std::sync::Arc;

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
) -> Result<()> {
    let quoter = crate::quoter::Quoter::new(rpc, input_mint, output_mint)?;
    let mut routes = quoter
        .lookup_routes2(input_amount, false, Slippage::FifteenBip, FeeBps::Zero)?
        .into_iter()
        .filter(|quote| {
            for market_info in quote.market_infos.iter() {
                if market_info.label.eq_ignore_ascii_case("orca (whirlpools)")
                    || market_info.label.eq_ignore_ascii_case("orca")
                    || market_info.label.eq_ignore_ascii_case("raydium")
                    || market_info.label.eq_ignore_ascii_case("saber")
                    || market_info.label.eq_ignore_ascii_case("mercurial")
                    || market_info.label.eq_ignore_ascii_case("lifinity")
                {
                    continue;
                } else {
                    println!("ignoring market {}", market_info.label);
                    return false;
                }
            }
            true
        })
        .collect::<Vec<_>>();
    if routes.is_empty() {
        return Err(anyhow!("failed to find any routes"));
    }
    let swap_route = std::mem::take(&mut routes[0]);

    #[cfg(test)]
    swap_route
        .market_infos
        .iter()
        .enumerate()
        .for_each(|(idx, minfo)| {
            println!(
                "market info {}. coin {} pc {}",
                idx, minfo.input_mint, minfo.output_mint
            );
        });

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
        swap,
        cleanup,
    } = swap_config;
    if setup.is_some() {
        log::info!("setup required");
    }

    for tx in [setup, Some(swap), cleanup]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .iter_mut()
    {
        // create the legacy and sanitized messages used for processesing
        let sanitized_msg = SanitizedMessage::Legacy(tx.message.clone());
        let mut instructions = Vec::with_capacity(tx.message.instructions.len());

        instructions.append(
            &mut tx
                .message
                .instructions
                .iter_mut()
                .map(|compiled_ix| {
                    Instruction::new_with_bytes(
                        *sanitized_msg
                            .get_account_key(compiled_ix.program_id_index.into())
                            .ok_or(InstructionError::MissingAccount)
                            .unwrap(),
                        &compiled_ix.data,
                        compiled_ix
                            .accounts
                            .iter()
                            .map(|account_index| {
                                let account_index = *account_index as usize;
                                Ok(AccountMeta {
                                    is_signer: sanitized_msg.is_signer(account_index),
                                    is_writable: sanitized_msg.is_writable(account_index),
                                    pubkey: *sanitized_msg
                                        .get_account_key(account_index)
                                        .ok_or(InstructionError::MissingAccount)?,
                                })
                            })
                            .collect::<Result<Vec<AccountMeta>, InstructionError>>()
                            .unwrap(),
                    )
                })
                .collect::<Vec<_>>(),
        );
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
                    if let Ok(args) = new_jupiter_swap_ix_data(ix.clone(), jup_ix, swap_input) {
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
                .map(|ix| ix.accounts.clone())
                .flatten()
                .collect::<Vec<_>>()[..],
        );
        let ix = Instruction {
            program_id: anyix_program,
            accounts,
            data: juper_swap_cpi::instructions::JupiterSwapArgs { input_data: any_ix }.data(),
        };
        let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));

        #[cfg(test)]
        println!(
            "encoded jupiter tx {}",
            serialize_and_encode(&tx, UiTransactionEncoding::Base64).unwrap()
        );

        tx.sign(&vec![&*payer], rpc.get_latest_blockhash()?);
        log::info!("sending jupiter swap ix");
        if skip_preflight {
            match rpc.send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    skip_preflight,
                    ..Default::default()
                },
            ) {
                Ok(sig) => log::info!("sent jupiter swap ix {}", sig),
                Err(err) => log::error!("failed to send jupiter swap ix {:#?}", err),
            }
        } else {
            match rpc.send_and_confirm_transaction(&tx) {
                Ok(sig) => log::info!("sent jupiter swap ix {}", sig),
                Err(err) => log::error!("failed to send jupiter swap ix {:#?}", err),
            }
        }
    }
    Ok(())
}

/// given an instruction from the jupiter swap api, encode the instruction
/// into the AnyIx format accepted by our vaults program
pub fn new_jupiter_swap_ix(
    swap_api_ix: Instruction,
    jup_ix: JupiterIx,
    swap_input: SwapInputs,
    anyix_program: Pubkey,
) -> Result<Instruction> {
    match jup_ix {
        JupiterIx::SetTokenLedger => {
            Ok(jup_ix.encode_token_ledger_ix(anyix_program, swap_api_ix.accounts))
        }
        _ => Ok(jup_ix.encode_swap_ix(swap_input, anyix_program, swap_api_ix.accounts)),
    }
}

pub struct AnyIxArgs {
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

/// given an instruction from the jupiter swap api, encode the instruction
/// into the AnyIx format accepted by our vaults program
pub fn new_jupiter_swap_ix_data(
    swap_api_ix: Instruction,
    jup_ix: JupiterIx,
    swap_input: SwapInputs,
) -> Result<AnyIxArgs> {
    let mut data: Vec<u8> = vec![jup_ix.into()];
    if jup_ix.ne(&JupiterIx::SetTokenLedger) {
        data.extend_from_slice(&swap_input.pack()[..]);
    }
    Ok(AnyIxArgs {
        data,
        accounts: swap_api_ix.accounts,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use anchor_lang::solana_program;
    use simplelog::*;
    use solana_sdk::signature::{self, Keypair};
    use solana_sdk::signer::Signer;
    use static_pubkey::static_pubkey;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::Arc;
    use std::{fs::File, str::FromStr};
    #[test]
    #[allow(unused_must_use)]
    fn test_new_anyix_swap() {
        TermLogger::init(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_location_level(LevelFilter::Error)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        );
        let rpc = Arc::new(RpcClient::new("https://ssc-dao.genesysgo.net".to_string()));
        let orca_mint = static_pubkey!("orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE");
        let msol_mint = static_pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
        let hbb_mint = static_pubkey!("HBB111SCo9jkCejsZfz8Ec8nH7T6THF8KEKSnvwT6XK6");
        let mnde_mint = static_pubkey!("MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey");
        let usdh_mint = static_pubkey!("USDH1SM1ojwWUga67PGrgFWUHibbjqMvuMaDkRJTgkX");
        /* simiulate a USDH-mSOL Whirlpool swap

        * this gives ORCA, HBB, MNDE rewards + fees
        * requires USDH-mSOL liquidity

        */

        let payer = Keypair::new();
        let vault_pda = static_pubkey!("663B7xaCqkFKeRKbWwbzcdXoeBLwNS1k5uDFVgUkZwh9");
        let vault = static_pubkey!("HvRLN4NtVojvM6MicHVnUWCfBZMVWY4mn147LitM27dE");
        let management = static_pubkey!("De74LEi2qAz5Lk8XTfm7dTRrhwpJVqbCjehLZSPzKfRN");
        let anyix_program = static_pubkey!("TLPv2haaXncGsurtzQb4rMnFvuPJto4mntAa51PidhD");
        new_anyix_swap(
            &rpc,
            &payer,
            anyix_program,
            management,
            vault,
            vault_pda,
            orca_mint,
            usdh_mint,
            1.0,
            false,
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        new_anyix_swap(
            &rpc,
            &payer,
            anyix_program,
            management,
            vault,
            vault_pda,
            mnde_mint,
            msol_mint,
            1.0,
            false,
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        new_anyix_swap(
            &rpc,
            &payer,
            anyix_program,
            management,
            vault,
            vault_pda,
            hbb_mint,
            usdh_mint,
            1.0,
            false,
        )
        .unwrap();
    }
}
