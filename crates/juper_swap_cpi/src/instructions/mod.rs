pub mod aldrin;
pub mod aldrin_v2;
pub mod cropper;
pub mod cykura;
pub mod lifinity;
pub mod mercurial;
pub mod raydium;
pub mod raydium_v2;
pub mod saber;
pub mod serum;
pub mod sighashes;
pub mod risk_check_and_fee;
pub mod token_ledger;
pub mod token_swap;
pub mod whirlpool;

use anchor_lang::prelude::*;

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

impl Side {
    /// returns true if the Side variant is an a => b swap
    pub fn a_to_b(&self) -> bool {
        self.eq(&Side::Ask)
    }
}

/// Instruction.
pub struct JupiterSwapArgs {
    pub input_data: Vec<u8>,
}
impl borsh::ser::BorshSerialize for JupiterSwapArgs
where
    Vec<u8>: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.input_data, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for JupiterSwapArgs
where
    Vec<u8>: borsh::BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            input_data: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
impl anchor_lang::InstructionData for JupiterSwapArgs {
    fn data(&self) -> Vec<u8> {
        let mut d = sighashes::JUPITER_SWAP_SIGHASH.to_vec();
        d.append(&mut self.try_to_vec().expect("Should always serialize"));
        d
    }
}
