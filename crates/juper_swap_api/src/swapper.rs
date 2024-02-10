use super::swap_types::SwapResponse;
use anyhow::{anyhow, Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_client::{rpc_config::RpcSendTransactionConfig};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    message::VersionedMessage,
    signature::{Keypair, Signature, Signer},
    transaction::VersionedTransaction,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Swapper {
    pub rpc: Arc<RpcClient>,
}

impl Swapper {
    pub fn new(rpc: Arc<RpcClient>) -> Swapper {
        Self { rpc }
    }
    pub async fn new_swap(
        self: &Arc<Self>,
        swap_response: SwapResponse,
        skip_preflight: bool,
        priority_fee: Option<f64>,
        cu_limit: Option<u32>,
        retries: Option<usize>,
        keypair_bytes: [u8; 64],
        fee_recipient: Option<Pubkey>,
        fee_amount: Option<u64>,
        input_mint: Pubkey,
    ) -> Result<Signature> {
        let priority_fee = if let Some(fee) = priority_fee {
            Some(prio_fee(fee))
        } else {
            None
        };

        let kp = Keypair::from_bytes(&keypair_bytes)?;
        Err(anyhow!("TODO"))
        /*let v0_msg = swap_response
            .new_v0_transaction(
                &self.rpc,
                kp.pubkey(),
                priority_fee,
                cu_limit,
                input_mint,
                fee_recipient,
                fee_amount,
            )
            .await?;
        //let v_tx = VersionedTransaction::try_new(VersionedMessage::V0(v0_msg), &vec![&kp])?;

        match self
            .rpc
            .send_transaction_with_config(
                &v_tx,
                RpcSendTransactionConfig {
                    skip_preflight,
                    max_retries: retries,
                    ..Default::default()
                },
            )
            .await
        {
            Ok(sig) => Ok(sig),
            Err(err) => return Err(anyhow!("failed execute swap {err:#?}")),
        }*/
    }
}

pub fn prio_fee(input: f64) -> u64 {
    spl_token::ui_amount_to_amount(input, 9)
}
