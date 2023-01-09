use borsh::{self, BorshSerialize};

/// Instruction.
pub struct RaydiumSwap {
    pub _in_amount: Option<u64>,
    pub _minimum_out_amount: u64,
    pub _platform_fee_bps: u8,
}
impl borsh::ser::BorshSerialize for RaydiumSwap
where
    Option<u64>: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    u8: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self._in_amount, writer)?;
        borsh::BorshSerialize::serialize(&self._minimum_out_amount, writer)?;
        borsh::BorshSerialize::serialize(&self._platform_fee_bps, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for RaydiumSwap
where
    Option<u64>: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    u8: borsh::BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            _in_amount: borsh::BorshDeserialize::deserialize(buf)?,
            _minimum_out_amount: borsh::BorshDeserialize::deserialize(buf)?,
            _platform_fee_bps: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
impl anchor_lang::InstructionData for RaydiumSwap {
    fn data(&self) -> Vec<u8> {
        let mut d = [177, 173, 42, 240, 184, 4, 124, 81].to_vec();
        d.append(&mut self.try_to_vec().expect("Should always serialize"));
        d
    }
}


impl anchor_lang::Discriminator for RaydiumSwap {
    const DISCRIMINATOR: [u8; 8] = super::sighashes::RAYDIUM_SWAP;
    fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }
}