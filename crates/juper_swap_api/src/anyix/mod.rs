pub mod swap;

use anchor_lang::solana_program::pubkey::Pubkey;

use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use juper_swap_cpi::{JupiterIx, SwapInputs};
use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;

use solana_sdk::instruction::Instruction;

use std::{collections::HashMap, sync::Arc};

/// todo move to regex
pub static DEFAULT_MARKET_LIST: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "orca (whirlpools)".to_string(),
        "orca".to_string(),
        "raydium".to_string(),
        "raydiumv2".to_string(),
        "saber".to_string(),
        "orca (whirlpools) (95%) + raydium (5%)".to_string(),
        "raydium (95%) + orca (5%)".to_string(),
        "orca (whirlpools) (85%) + orca (15%)".to_string(),
        "orca (95%) + raydium (5%)".to_string(),
        "cykura".to_string()
        //"mercurial".to_string(),
        //"lifinity".to_string(),
    ]
});

/// wraps the instruction data, and instruction accounts required by an AnyIx instruction
pub struct AnyIxArgs {
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
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

/// given an instruction `ix`, iterate over all all accounts,
/// applying `modify_fn` which if needed replace the account output with
/// one matching a predefined ruleset
pub fn replace_accounts(
    ix: &mut Instruction,
    rpc: &Arc<RpcClient>,
    // returns None if no modification needed
    modify_fn: &mut dyn FnMut(
        // the account being checked for replacement
        &AccountMeta,
        // the rpc used to check on-chain state
        &Arc<RpcClient>,
        // a hashmap used to map the account being replaced, with the account replacing it
        &HashMap<Pubkey, Pubkey>,
    ) -> Option<AccountMeta>,
    replacements: &HashMap<Pubkey, Pubkey>,
) -> Result<()> {
    for account in ix.accounts.iter_mut() {
        if let Some(new_acct) = modify_fn(account, rpc, replacements) {
            log::warn!("replacing {} with {}", account.pubkey, new_acct.pubkey);
            *account = new_acct;
        }
    }
    Ok(())
}

/// used as a `modify_fn` parameter in `replace_accounts`, this
/// performs a basic replacement operation, matching on the account
/// address
pub fn replace_by_account_pubkey(
    account: &AccountMeta,
    _rpc: &Arc<RpcClient>,
    replacements: &HashMap<Pubkey, Pubkey>,
) -> Option<AccountMeta> {
    replacements.get(&account.pubkey).map(|new| AccountMeta {
        pubkey: *new,
        is_writable: account.is_writable,
        is_signer: account.is_signer,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::anyix::test::solana_program::message::SanitizedMessage;
    use crate::slippage::Slippage;
    use anchor_lang::solana_program;
    use simplelog::*;
    use solana_sdk::instruction::InstructionError;
    use solana_sdk::signature::Keypair;
    use solana_sdk::transaction::Transaction;
    use static_pubkey::static_pubkey;
    use std::collections::HashMap;

    use std::sync::Arc;

    pub fn derive_tokena_compound_queue(vault: Pubkey, mint: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"tokena_compound_queue", vault.as_ref(), mint.as_ref()],
            &static_pubkey!("TLPv2haaXncGsurtzQb4rMnFvuPJto4mntAa51PidhD"),
        )
    }

    pub fn derive_tokenb_compound_queue(vault: Pubkey, mint: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"tokenb_compound_queue", vault.as_ref(), mint.as_ref()],
            &static_pubkey!("TLPv2haaXncGsurtzQb4rMnFvuPJto4mntAa51PidhD"),
        )
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_anyix_swap_override() {
        TermLogger::init(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_location_level(LevelFilter::Error)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        );
        let rpc = Arc::new(RpcClient::new("https://ssc-dao.genesysgo.net".to_string()));
        let _orca_mint = static_pubkey!("orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE");
        let _msol_mint = static_pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
        let _hbb_mint = static_pubkey!("HBB111SCo9jkCejsZfz8Ec8nH7T6THF8KEKSnvwT6XK6");
        let mnde_mint = static_pubkey!("MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey");
        let usdh_mint = static_pubkey!("USDH1SM1ojwWUga67PGrgFWUHibbjqMvuMaDkRJTgkX");
        let vault_pda = static_pubkey!("663B7xaCqkFKeRKbWwbzcdXoeBLwNS1k5uDFVgUkZwh9");
        let vault = static_pubkey!("HvRLN4NtVojvM6MicHVnUWCfBZMVWY4mn147LitM27dE");
        let _management = static_pubkey!("De74LEi2qAz5Lk8XTfm7dTRrhwpJVqbCjehLZSPzKfRN");
        let _anyix_program = static_pubkey!("TLPv2haaXncGsurtzQb4rMnFvuPJto4mntAa51PidhD");
        let b64_encoded_swap_ix = base64::decode("AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAoaudEwYcD83iSmOdiCYgl0FbE1EY/HTjC9O5vVR1aEHFWqc2Opgg73NeMj+N8A8YBI7KBlgAadkiYTWIQY7+9V8RhD6EtkoqBLhz59tziCckQWnJgl6UD5U8gAbmrlZLYAS5TMIJJnkcTF2Y9tnZJBMYjEdZLTelaWUxXjYxc1LQBy6joJxxNxIhqReCIF6ipi6l8dFod/NVBGlOa9DSJu5ISU5qD0vvrQhq+nyqyPEfzUf6sPd2hWARpSfIxTXxCGUV3Qo2oarR8RfanRO45fsiz96o0HIJ1mCfSLByJGBWQLj3UTm7d3XtPJYOBovoo6CwRU3hvpqEJYQE/IzLTCZ1waD1HOYC2xb6ZMp5eamugBnFs8axvo2gd01mOANYU53dHPHkhuGADDuqPZzbztsigJaTDSQgZyxNrWOAy/zmxXYyJa5D0osAqQhhSw0/YyqDhnIGj0GwDx3EdYMq7gmzQ75dsN/tO3arD+4cQKCwImHdTmOLddU/PySdwwBfo74xNweqSUKpWIXr/IcQHRrcRUO2enkrQ5Z80qSH/OUJKWKpuQ7lVrgHG91hWFUjOg8vE3WvyY/WShXVHKFExQ0POdIYrSqXHH5oP8cJLTXQbhX15dEO6iKalbaYnCU2fyQ9T66GpX3xFpsi1C4nTOMgvpYSQbxTsZEDouboxsAZm7zZUMob9Z6rKfT5aGxqXL6rHuJlQvFBgJ1Q9HioX3PftqSb72Tp8fjTN0jXGTukPmEi7RgGqM+MLi6vWxymkVBHnVH6nNSvb3qwqwbkgtxkwyjapZJyq+L06YgldeDnh+VHcaV6bxTKnkAtVK7kX3N4rKNlx7Fpp+yD9RgrKY8Abd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpGu2oNYDga4vix7P151kKaci+vDfG0APaJSmfnKJlpZ8nqtoym+YRqdZn6qX1bDrJAVznq35qhT3m2r7+nOVT4g4DaF+OkJBT5FgSHGb1p2rtx3BqoRyC+KqVKo8reHmpK9feXDkiJL2QzliBZ93nohLYkTD7z3ZjZmE+kHJx8sMGvwf3Oo0dtK8U/tuext6Xa3u+NU+95O6RRSyGRvtAdAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARkeABAREgECExQVFgMEBQYCBwgXFAMJCgsCDA0ODxgBLnTPAMT8ePMSIgAAAAMBEQkCCw0LAEBCDwAAAAAAAAAAAAAAAAAIHZsMAAAAAAA=").unwrap();
        let mut orig_txn: Transaction = bincode::deserialize(&b64_encoded_swap_ix[..]).unwrap();
        let sanitized_msg = SanitizedMessage::Legacy(orig_txn.message().clone());
        let mut orig_instructions = Vec::with_capacity(12);
        orig_instructions.append(
            &mut orig_txn
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
                                        .ok_or(InstructionError::MissingAccount)
                                        .unwrap(),
                                })
                            })
                            .collect::<Result<Vec<AccountMeta>, InstructionError>>()
                            .unwrap(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        let orig_instruction = orig_instructions[0].clone();
        let mut new_instruction = orig_instruction.clone();
        let mut replacements = HashMap::default();
        replacements.insert(
            spl_associated_token_account::get_associated_token_address(&vault_pda, &usdh_mint),
            derive_tokena_compound_queue(vault, usdh_mint).0,
        );
        replacements.insert(
            spl_associated_token_account::get_associated_token_address(&vault_pda, &mnde_mint),
            derive_tokenb_compound_queue(vault, mnde_mint).0,
        );
        replace_accounts(
            &mut new_instruction,
            &rpc,
            &mut replace_by_account_pubkey,
            &replacements,
        )
        .unwrap();
        assert_ne!(new_instruction, orig_instruction);

        for (idx, (new, old)) in new_instruction
            .accounts
            .iter()
            .zip(orig_instruction.accounts)
            .enumerate()
        {
            if idx == 21 {
                assert_eq!(
                    new.pubkey.to_string().as_str(),
                    "ApFizqZ9hfyDEVMtMGvTVBE9hyEV7EwRvLninB4K919n"
                );
                assert_eq!(
                    old.pubkey.to_string().as_str(),
                    "6t89P5TdjKv5L9JHE8bVfb2APx8jqvxfyVytFRDRaBPG"
                );
            }
            log::info!(
                "idx {}, new_key {}, old_key {}",
                idx,
                new.pubkey,
                old.pubkey
            );
        }
    }
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
        let mut replacements = HashMap::default();

        replacements.insert(
            spl_associated_token_account::get_associated_token_address(&vault_pda, &usdh_mint),
            derive_tokena_compound_queue(vault, usdh_mint).0,
        );
        replacements.insert(
            spl_associated_token_account::get_associated_token_address(&vault_pda, &mnde_mint),
            derive_tokenb_compound_queue(vault, mnde_mint).0,
        );

        super::swap::new_anyix_swap(
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
            2,
            None,
            &replacements,
            Slippage::TwentyBip,
        );

        super::swap::new_anyix_swap(
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
            2,
            None,
            &Default::default(),
            Slippage::TwentyBip,
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        super::swap::new_anyix_swap(
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
            2,
            Some(DEFAULT_MARKET_LIST.to_vec()),
            &Default::default(),
            Slippage::TwentyBip,
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        super::swap::new_anyix_swap(
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
            2,
            None,
            &Default::default(),
            Slippage::TwentyBip,
        );
    }
}
