use anchor_lang::prelude::AccountMeta;
use anyhow::{anyhow, Result};
use solana_sdk::message::SanitizedMessage;
use solana_sdk::{
    instruction::{Instruction, InstructionError},
    transaction::Transaction,
};

/// decompiles a transaction into it's individual instructions, sorted
/// by execution order
pub fn decompile_transaction_instructions(mut tx: Transaction) -> Result<Vec<Instruction>> {
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
    Ok(instructions)
}
