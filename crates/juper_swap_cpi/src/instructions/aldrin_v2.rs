use crate::Side;
use anchor_lang::Discriminator;
use borsh::{self, BorshSerialize};
/// Instruction.
pub struct AldrinV2Swap {
    pub _in_amount: Option<u64>,
    pub _minimum_out_amount: u64,
    pub _side: Side,
    pub _platform_fee_bps: u8,
}
impl borsh::ser::BorshSerialize for AldrinV2Swap
where
    Option<u64>: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    Side: borsh::ser::BorshSerialize,
    u8: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self._in_amount, writer)?;
        borsh::BorshSerialize::serialize(&self._minimum_out_amount, writer)?;
        borsh::BorshSerialize::serialize(&self._side, writer)?;
        borsh::BorshSerialize::serialize(&self._platform_fee_bps, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for AldrinV2Swap
where
    Option<u64>: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    Side: borsh::BorshDeserialize,
    u8: borsh::BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            _in_amount: borsh::BorshDeserialize::deserialize(buf)?,
            _minimum_out_amount: borsh::BorshDeserialize::deserialize(buf)?,
            _side: borsh::BorshDeserialize::deserialize(buf)?,
            _platform_fee_bps: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
impl anchor_lang::InstructionData for AldrinV2Swap {
    fn data(&self) -> Vec<u8> {
        let mut d = super::sighashes::ALDRIN_V2_SWAP.to_vec();
        d.append(&mut self.try_to_vec().expect("Should always serialize"));
        d
    }
}
impl anchor_lang::Discriminator for AldrinV2Swap {
    const DISCRIMINATOR: [u8; 8] = super::sighashes::ALDRIN_V2_SWAP;
    fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }
}