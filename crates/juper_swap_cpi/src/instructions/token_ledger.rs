use borsh::{self, BorshSerialize};

/// Instruction.
pub struct SetTokenLedger;
impl borsh::ser::BorshSerialize for SetTokenLedger {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        _writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for SetTokenLedger {
    fn deserialize(_buf: &mut &[u8]) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {})
    }
}
impl anchor_lang::InstructionData for SetTokenLedger {
    fn data(&self) -> Vec<u8> {
        let mut d = [228, 85, 185, 112, 78, 79, 77, 2].to_vec();
        d.append(&mut self.try_to_vec().expect("Should always serialize"));
        d
    }
}



impl anchor_lang::Discriminator for SetTokenLedger {
    const DISCRIMINATOR: [u8; 8] = super::sighashes::SET_TOKEN_LEDGER;
    fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }
}