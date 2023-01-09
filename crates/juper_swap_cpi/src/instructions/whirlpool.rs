use borsh::{self, BorshSerialize};

/// Instruction.
pub struct WhirlpoolSwap {
    pub _in_amount: Option<u64>,
    pub _minimum_out_amount: u64,
    pub _a_to_b: bool,
    pub _platform_fee_bps: u8,
}
impl borsh::ser::BorshSerialize for WhirlpoolSwap
where
    Option<u64>: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
    u8: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self._in_amount, writer)?;
        borsh::BorshSerialize::serialize(&self._minimum_out_amount, writer)?;
        borsh::BorshSerialize::serialize(&self._a_to_b, writer)?;
        borsh::BorshSerialize::serialize(&self._platform_fee_bps, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for WhirlpoolSwap
where
    Option<u64>: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
    u8: borsh::BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            _in_amount: borsh::BorshDeserialize::deserialize(buf)?,
            _minimum_out_amount: borsh::BorshDeserialize::deserialize(buf)?,
            _a_to_b: borsh::BorshDeserialize::deserialize(buf)?,
            _platform_fee_bps: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
impl anchor_lang::InstructionData for WhirlpoolSwap {
    fn data(&self) -> Vec<u8> {
        let mut d = [123, 229, 184, 63, 12, 0, 92, 145].to_vec();
        d.append(&mut self.try_to_vec().expect("Should always serialize"));
        d
    }
}

impl anchor_lang::Discriminator for WhirlpoolSwap {
    const DISCRIMINATOR: [u8; 8] = super::sighashes::WHIRLPOOL_SWAP;
    fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }
}