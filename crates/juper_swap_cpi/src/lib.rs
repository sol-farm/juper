/// a lightweight crate that can be used both for Jupiter CPI, as well as for
/// implementing, and using proxied jupiter swaps via AnyIx
pub mod accounts;
pub mod instructions;

use solana_program::{
    self,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};
use static_pubkey::static_pubkey;

use instructions::Side;

/// the program id of the v3 jupiter aggregator
pub const JUPITER_V3_AGG_ID: Pubkey = static_pubkey!("JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph");
/// the program id of the v6 jupiter aggregator
pub const JUPITER_V6_AGG_ID: Pubkey = static_pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
pub const JUPITER_AGG_IDS: [Pubkey; 2] = [JUPITER_V3_AGG_ID, JUPITER_V6_AGG_ID];

/// alias to satisfy anchor codegen requirements
///pub const ID: Pubkey = JUPITER_V6_AGG_ID;
use anchor_lang::{prelude::*, InstructionData};

use solana_program::{instruction::Instruction, program_pack::Pack};
use std::collections::BTreeMap;

use crate::instructions::v6_ixs::V6Instructions;

/// The reference implementation of the AnyIx instruction parser for use with
/// the juper_swap_cpi crate. It performs basic access controls, ensuring
/// the provided jupiter progrma account is the V3 aggregator, and that all
/// input and output token accounts are owned by the `wanted_toke_owner`.
///
/// If you need any additional validations, you should implement your own function
pub fn process_instructions<'info>(
    // the account address that should own the token accounts which will receive
    // the outputs and inputs of the swaps
    wanted_token_owner: Pubkey,
    jupiter_program_account: &AccountInfo<'info>,
    remaining_accounts: &mut [AccountInfo<'info>],
    data: &[u8],
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    // ensure an acceptable programId is being used
    assert!(JUPITER_AGG_IDS.contains(jupiter_program_account.key));
    if jupiter_program_account.key.eq(&JUPITER_V6_AGG_ID) {
        return process_v6_swap(&remaining_accounts, jupiter_program_account, Default::default(), Default::default(), Default::default(), data, seeds);
    }

    let any_ix = anyix::AnyIx::unpack(data)?;
    let anyix::AnyIx {
        num_instructions,
        instruction_data_sizes: _,
        instruction_datas,
        instruction_account_counts,
    } = any_ix;
    let mut offset = 0;
    for idx in 0..num_instructions {
        let accounts = &remaining_accounts[offset as usize..];
        offset += instruction_account_counts[idx as usize] as usize;

        // the first element of the data slice is going to be the JupiterIx variant
        let jupiter_ix: JupiterIx = From::from(instruction_datas[idx as usize][0]);
        let swap_inputs = SwapInputs::new().unpack(&instruction_datas[idx as usize][1..]);
        jupiter_ix.execute(
            accounts,
            seeds,
            swap_inputs.side(),
            swap_inputs.input_amount,
            swap_inputs.min_output,
            wanted_token_owner,
            instruction_account_counts[idx as usize] as usize,
        );
    }
    Ok(())
}

/// Decodes the instruction data for a single instruction included within the transactions
/// returned by jupiter's swap api. The decoded instruction data is then parsed into
/// the market the swap is being routed on, along with the swap input values
pub fn decode_jupiter_instruction(input: &[u8]) -> Result<(JupiterIx, SwapInputs)> {
    if input.len() > 8 {
        let (ix_data, inputs) = input.split_at(8);
        let jupiter_ix: JupiterIx = TryFrom::try_from(ix_data)?;
        Ok((jupiter_ix, jupiter_ix.get_swap_inputs(inputs)?))
    } else {
        let jupiter_ix: JupiterIx = TryFrom::try_from(input)?;
        Ok((
            jupiter_ix,
            SwapInputs {
                input_amount: None,
                min_output: 0,
                side: 0,
            },
        ))
    }
}

//
pub struct V6SwapVerificationOpts {
    pub wanted_transfer_authority: Pubkey,
    pub wanted_user_source_token_account_owner: Pubkey,
    pub wanted_user_destination_token_account_owner: Pubkey,
}

pub fn process_v6_swap<'info>(
    remaining_accounts: &[AccountInfo],
    jupiter_program: &AccountInfo<'info>,
    wanted_transfer_authority: Pubkey,
    want_user_source_token_account_owner: Pubkey,
    wanted_user_destination_token_account_owner: Pubkey,
    data: &[u8],
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    // ensure we are only swapping against the v6 program id
    assert!(jupiter_program.key.eq(&JUPITER_V6_AGG_ID));
    // verify the instruction sighash is a genuine one and expected
    let ix = V6Instructions::try_from(&data[0..8].try_into().unwrap()).unwrap();
    // validate the input accounts, notably signer, source+destination user token account owners
    ix.validate_accounts(remaining_accounts, wanted_transfer_authority, want_user_source_token_account_owner, wanted_user_destination_token_account_owner);
    let accounts: Vec<AccountMeta> = remaining_accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: *acc.key,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let accounts_infos: Vec<AccountInfo> = remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    // TODO: Check the first 8 bytes. Only Jupiter Route CPI allowed.
    if let Some(seeds) = seeds {
        invoke_signed(
            &Instruction {
                program_id: *jupiter_program.key,
                accounts,
                data: data.to_vec(),
            },
            &accounts_infos,
            seeds,
        )?;
    } else {
        invoke(
            &Instruction {
                program_id: *jupiter_program.key,
                accounts,
                data: data.to_vec(),
            },
            &accounts_infos,
        )?;
    }
    Ok(())
}

/// Wraps the input values for a given swap
#[derive(Clone, Copy, Default, Debug)]
pub struct SwapInputs {
    pub input_amount: Option<u64>,
    pub min_output: u64,
    pub side: u8,
}

impl SwapInputs {
    pub fn new() -> Self {
        Self {
            input_amount: None,
            min_output: 0,
            side: 0,
        }
    }
    pub fn side(&self) -> Side {
        if self.side == 0 {
            Side::Ask
        } else {
            Side::Bid
        }
    }
    /// Unpacks the given `data` buffer into self
    pub fn unpack(&mut self, data: &[u8]) -> Self {
        let data_len = data.len();

        #[cfg(test)]
        println!("data len {}", data_len);

        if data_len == 17 {
            let (input_amount, rest) = data.split_at(8);
            let (min_output, rest) = rest.split_at(8);
            self.input_amount = Some(u64::try_from_slice(input_amount).unwrap());
            self.min_output = u64::try_from_slice(min_output).unwrap();
            self.side = rest[0];
        } else if data_len == 16 {
            // we can probably remove this case
            let (input_amount, min_output) = data.split_at(8);
            self.input_amount = Some(u64::try_from_slice(input_amount).unwrap());
            self.min_output = u64::try_from_slice(min_output).unwrap();
        } else if data_len == 9 {
            self.min_output = u64::try_from_slice(&data[0..8]).unwrap();
            self.side = data[8];
        } else if data_len == 8 {
            // we can probably remove this case
            self.min_output = u64::try_from_slice(data).unwrap();
        }
        *self
    }
    /// serializes the swap inputs
    pub fn pack(self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(17);
        if let Some(input_amount) = self.input_amount {
            buffer.extend_from_slice(&input_amount.to_le_bytes()[..]);
        }
        buffer.extend_from_slice(&self.min_output.to_le_bytes()[..]);
        buffer.push(self.side);
        buffer
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum JupiterIx {
    TokenSwap = 0_u8,
    AldrinV2Swap = 1_u8,
    CropperTokenSwap = 2_u8,
    CykuraTokenSwap = 3_u8,
    LifinityTokenSwap = 4_u8,
    MercurialExchange = 5_u8,
    RaydiumSwap = 6_u8,
    RaydiumSwapV2 = 7_u8,
    Whirlpool = 8_u8,
    Serum = 9_u8,
    Saber = 10_u8,
    SetTokenLedger = 11_u8,
    RiskCheckAndFee = 12_u8,
    AldrinSwap = 13_u8,
}

impl From<u8> for JupiterIx {
    fn from(input: u8) -> Self {
        match input {
            0 => Self::TokenSwap,
            1 => Self::AldrinV2Swap,
            2 => Self::CropperTokenSwap,
            3 => Self::CykuraTokenSwap,
            4 => Self::LifinityTokenSwap,
            5 => Self::MercurialExchange,
            6 => Self::RaydiumSwap,
            7 => Self::RaydiumSwapV2,
            8 => Self::Whirlpool,
            9 => Self::Serum,
            10 => Self::Saber,
            11 => Self::SetTokenLedger,
            12 => Self::RiskCheckAndFee,
            13 => Self::AldrinSwap,
            _ => panic!("invalid input {}", input),
        }
    }
}

impl From<JupiterIx> for u8 {
    fn from(ix: JupiterIx) -> Self {
        match ix {
            JupiterIx::TokenSwap => 0,
            JupiterIx::AldrinV2Swap => 1,
            JupiterIx::CropperTokenSwap => 2,
            JupiterIx::CykuraTokenSwap => 3,
            JupiterIx::LifinityTokenSwap => 4,
            JupiterIx::MercurialExchange => 5,
            JupiterIx::RaydiumSwap => 6,
            JupiterIx::RaydiumSwapV2 => 7,
            JupiterIx::Whirlpool => 8,
            JupiterIx::Serum => 9,
            JupiterIx::Saber => 10,
            JupiterIx::SetTokenLedger => 11,
            JupiterIx::RiskCheckAndFee => 12,
            JupiterIx::AldrinSwap => 13,
        }
    }
}

/// given what is presumed to be anchor program instruction data
/// which is equal to or greater than  8 bytes, determine which
/// of the JupiterIx variants this instruction data is for.
///
/// while this can technically match any anchor program instruction data
/// it's meant for use with jupiter instructions, usage with non jupiter instructions
/// is unsupported
impl TryFrom<&[u8]> for JupiterIx {
    type Error = ProgramError;
    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        use instructions::sighashes::*;
        if value.len() < 8 {
            msg!("instruction data to small {}", value.len());
            return Err(ProgramError::AccountDataTooSmall);
        }
        let ix_data = &value[0..8];
        if ix_data.eq(&MERCURIAL_EXCHANGE) {
            Ok(Self::MercurialExchange)
        } else if ix_data.eq(&SABER_SWAP) {
            Ok(Self::Saber)
        } else if ix_data.eq(&SERUM_SWAP) {
            Ok(Self::Serum)
        } else if ix_data.eq(&TOKEN_SWAP) {
            Ok(Self::TokenSwap)
        } else if ix_data.eq(&STEP_TOKEN_SWAP) {
            msg!("step unsupported");
            Err(ProgramError::InvalidInstructionData)
        } else if ix_data.eq(&CROPPER_TOKEN_SWAP) {
            Ok(Self::CropperTokenSwap)
        } else if ix_data.eq(&RAYDIUM_SWAP) {
            Ok(Self::RaydiumSwap)
        } else if ix_data.eq(&RAYDIUM_SWAP_V2) {
            Ok(Self::RaydiumSwapV2)
        } else if ix_data.eq(&CREMA_TOKEN_SWAP) {
            msg!("crema unsupported");
            Err(ProgramError::InvalidInstructionData)
        } else if ix_data.eq(&LIFINITY_TOKEN_SWAP) {
            Ok(Self::LifinityTokenSwap)
        } else if ix_data.eq(&CYKURA_SWAP) {
            Ok(Self::CykuraTokenSwap)
        } else if ix_data.eq(&WHIRLPOOL_SWAP) {
            Ok(Self::Whirlpool)
        } else if ix_data.eq(&SET_TOKEN_LEDGER) {
            Ok(Self::SetTokenLedger)
        } else if ix_data.eq(&RISK_CHECK_AND_FEE) {
            Ok(Self::RiskCheckAndFee)
        } else if ix_data.eq(&ALDRIN_V2_SWAP) {
            Ok(Self::AldrinV2Swap)
        } else {
            msg!("invalid jupiter ix {:#?}", value);
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

impl JupiterIx {
    /// given the buffer `data`, deserialize into swap inputs
    /// for a protocol as determined by the variant of self
    pub fn get_swap_inputs(&self, data: &[u8]) -> Result<SwapInputs> {
        match self {
            JupiterIx::TokenSwap => match instructions::token_swap::TokenSwap::try_from_slice(data)
            {
                Ok(input) => Ok(SwapInputs {
                    input_amount: input._in_amount,
                    min_output: input._minimum_out_amount,
                    side: 0,
                }),
                Err(err) => {
                    msg!("failed to parse token swap {:#?}", err);
                    Err(ProgramError::InvalidInstructionData.into())
                }
            },
            JupiterIx::AldrinV2Swap => {
                match instructions::aldrin_v2::AldrinV2Swap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse aldrinv2 swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::AldrinSwap => match instructions::aldrin::AldrinSwap::try_from_slice(data) {
                Ok(input) => Ok(SwapInputs {
                    input_amount: input._in_amount,
                    min_output: input._minimum_out_amount,
                    side: 0,
                }),
                Err(err) => {
                    msg!("failed to parse aldrin swap {:#?}", err);
                    Err(ProgramError::InvalidInstructionData.into())
                }
            },
            JupiterIx::CropperTokenSwap => {
                match instructions::cropper::CropperTokenSwap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse cropper swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::CykuraTokenSwap => {
                match instructions::cykura::CykuraSwap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse cykrua swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::LifinityTokenSwap => {
                match instructions::lifinity::LifinityTokenSwap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse lifinity swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::MercurialExchange => {
                match instructions::mercurial::MercurialExchange::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse mercurial swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::RaydiumSwap => {
                match instructions::raydium::RaydiumSwap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse ray swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::RaydiumSwapV2 => {
                match instructions::raydium_v2::RaydiumSwapV2::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse rayv2 swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::Whirlpool => {
                match instructions::whirlpool::WhirlpoolSwap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse whirlpool swap {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
            JupiterIx::Serum => match instructions::serum::SerumSwap::try_from_slice(data) {
                Ok(input) => Ok(SwapInputs {
                    input_amount: input._in_amount,
                    min_output: input._minimum_out_amount,
                    side: 0,
                }),
                Err(err) => {
                    msg!("failed to parse serum swap {:#?}", err);
                    Err(ProgramError::InvalidInstructionData.into())
                }
            },
            JupiterIx::Saber => match instructions::saber::SaberSwap::try_from_slice(data) {
                Ok(input) => Ok(SwapInputs {
                    input_amount: input._in_amount,
                    min_output: input._minimum_out_amount,
                    side: 0,
                }),
                Err(err) => {
                    msg!("failed to parse saber swap {:#?}", err);
                    Err(ProgramError::InvalidInstructionData.into())
                }
            },
            JupiterIx::SetTokenLedger => Ok(Default::default()),
            JupiterIx::RiskCheckAndFee => {
                match instructions::risk_check_and_fee::RiskCheckAndFee::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: None,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("risk check and fee failed {:#?}", err);
                        Err(ProgramError::InvalidInstructionData.into())
                    }
                }
            }
        }
    }
    /// Executes a single swap via the jupiter aggregator program
    /// validating that the input/output token accounts are owned
    /// by `signer`.
    ///
    /// A `None` value for `input` is overriden with the token amount
    /// value of the source token account being used in the swap
    pub fn execute<'info>(
        &self,
        mut accounts: &[AccountInfo<'info>],
        seeds: Option<&[&[&[u8]]]>,
        side: Side,
        input: Option<u64>,
        min_output: u64,
        signer: Pubkey,
        num_accounts: usize,
    ) {
        let (mut ix, account_infos, skip_signer) = match self {
            Self::TokenSwap => {
                msg!("processing token swap");
                let mer_swap = accounts::TokenSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account =
                    spl_token::state::Account::unpack(&mer_swap.source.data.borrow()).unwrap();
                let dest_token_account =
                    spl_token::state::Account::unpack(&mer_swap.destination.data.borrow()).unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::token_swap::TokenSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::AldrinV2Swap => {
                msg!("processing aldrinv2 swap");
                let mer_swap = accounts::AldrinV2Swap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account = spl_token::state::Account::unpack(
                    &mer_swap.user_base_token_account.data.borrow(),
                )
                .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &mer_swap.user_quote_token_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::aldrin_v2::AldrinV2Swap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                    _side: side,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::AldrinSwap => {
                msg!("processing aldrin swap");
                let mer_swap = accounts::AldrinSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account = spl_token::state::Account::unpack(
                    &mer_swap.user_base_token_account.data.borrow(),
                )
                .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &mer_swap.user_quote_token_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::aldrin::AldrinSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                    _side: side,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::CropperTokenSwap => {
                msg!("processing cropper swap");
                let mer_swap = accounts::CropperTokenSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account =
                    spl_token::state::Account::unpack(&mer_swap.source.data.borrow()).unwrap();
                let dest_token_account =
                    spl_token::state::Account::unpack(&mer_swap.destination.data.borrow()).unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::cropper::CropperTokenSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::CykuraTokenSwap => {
                msg!("processing cykura swap");
                let mer_swap = accounts::CykuraSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account =
                    spl_token::state::Account::unpack(&mer_swap.input_token_account.data.borrow())
                        .unwrap();
                let dest_token_account =
                    spl_token::state::Account::unpack(&mer_swap.output_token_account.data.borrow())
                        .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::cykura::CykuraSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::LifinityTokenSwap => {
                msg!("processing lifinity swap");
                let mer_swap = accounts::LifinityTokenSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account =
                    spl_token::state::Account::unpack(&mer_swap.source_info.data.borrow()).unwrap();
                let dest_token_account =
                    spl_token::state::Account::unpack(&mer_swap.destination_info.data.borrow())
                        .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::lifinity::LifinityTokenSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::MercurialExchange => {
                msg!("processing mercurial swap");
                let mer_swap = accounts::MercurialExchange::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account =
                    spl_token::state::Account::unpack(&mer_swap.source_token_account.data.borrow())
                        .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &mer_swap.destination_token_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::mercurial::MercurialExchange {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut meta_accounts = mer_swap.to_account_metas(None);
                if meta_accounts.len() < num_accounts {
                    let diff = num_accounts.checked_sub(accounts.len()).unwrap();
                    meta_accounts
                        .extend_from_slice(&take_accounts_into_metas(&mut accounts, diff)[..]);
                }
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: meta_accounts,
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::RaydiumSwap => {
                msg!("processing raydium swap");
                let ray_swap = accounts::RaydiumSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account = spl_token::state::Account::unpack(
                    &ray_swap.user_source_token_account.data.borrow(),
                )
                .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &ray_swap.user_destination_token_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::raydium::RaydiumSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: ray_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, ray_swap.to_account_infos(), false)
            }
            Self::RaydiumSwapV2 => {
                msg!("processing raydiumv2 swap");
                let ray_swap = accounts::RaydiumSwapV2::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account = spl_token::state::Account::unpack(
                    &ray_swap.user_source_token_account.data.borrow(),
                )
                .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &ray_swap.user_destination_token_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::raydium_v2::RaydiumSwapV2 {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: ray_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, ray_swap.to_account_infos(), false)
            }
            Self::Whirlpool => {
                msg!("processing whirlpool swap");
                let whirlpool_swap = accounts::WhirlpoolSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account = spl_token::state::Account::unpack(
                    &whirlpool_swap.token_owner_account_a.data.borrow(),
                )
                .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &whirlpool_swap.token_owner_account_b.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::whirlpool::WhirlpoolSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _a_to_b: side.a_to_b(),
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: whirlpool_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, whirlpool_swap.to_account_infos(), false)
            }
            Self::Serum => {
                msg!("processing serum swap");
                let serum_swap = accounts::SerumSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                // serum swaps require open order accounts, which we implement via our serum trade account
                // therefore we need to perform a bit different validation
                //
                // therefore dont bother validating the order payer token account, but validate the wallet accounts instead

                {
                    let source_token_account =
                        spl_token::state::Account::unpack(&serum_swap.coin_wallet.data.borrow())
                            .unwrap();
                    let dest_token_account =
                        spl_token::state::Account::unpack(&serum_swap.pc_wallet.data.borrow())
                            .unwrap();
                    assert!(source_token_account.owner.eq(&signer));
                    assert!(dest_token_account.owner.eq(&signer));
                }
                let source_token_account = spl_token::state::Account::unpack(
                    &serum_swap.order_payer_token_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                let ix_data = instructions::serum::SerumSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _side: side,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: serum_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, serum_swap.to_account_infos(), false)
            }
            Self::SetTokenLedger => {
                msg!("processing set token ledger");
                let token_ledger = accounts::SetTokenLedger::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let token_account =
                    spl_token::state::Account::unpack(&token_ledger.token_account.data.borrow())
                        .unwrap();
                assert!(token_account.owner.eq(&signer));
                let ix_data = instructions::token_ledger::SetTokenLedger {}.data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: token_ledger.to_account_metas(None),
                    data: ix_data,
                };
                (ix, token_ledger.to_account_infos(), true)
            }
            Self::Saber => {
                msg!("processing saber swap");
                let saber_swap = accounts::SaberSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let source_token_account =
                    spl_token::state::Account::unpack(&saber_swap.input_user_account.data.borrow())
                        .unwrap();
                let dest_token_account = spl_token::state::Account::unpack(
                    &saber_swap.output_user_account.data.borrow(),
                )
                .unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::saber::SaberSwap {
                    _in_amount: Some(input.unwrap_or(source_token_account.amount)),
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: saber_swap.to_account_metas(None),
                    data: ix_data,
                };
                (ix, saber_swap.to_account_infos(), false)
            }
            Self::RiskCheckAndFee => {
                msg!("processing risk check and fee");
                let risk_check = accounts::RiskCheckAndFee::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let user_dest_account = spl_token::state::Account::unpack(
                    &risk_check.user_destination_token_account.data.borrow(),
                )
                .unwrap();
                assert!(user_dest_account.owner.eq(&signer));
                assert!(risk_check.user_transfer_authority.key.eq(&signer));
                let ix_data = instructions::risk_check_and_fee::RiskCheckAndFee {
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: risk_check.to_account_metas(None),
                    data: ix_data,
                };
                (ix, risk_check.to_account_infos(), true)
            }
        };
        if !skip_signer {
            ix.accounts.iter_mut().for_each(|acct| {
                // we need to do this because the encoded transactions
                // from anyix will override the signer field
                if acct.pubkey.eq(&signer) {
                    acct.is_signer = true;
                }
            });
        }
        if !skip_signer {
            if let Some(seeds) = seeds {
                msg!("invoking signed");
                anchor_lang::solana_program::program::invoke_signed(&ix, &account_infos[..], seeds)
                    .unwrap();
            }
        } else {
            msg!("invoking unsigned");
            anchor_lang::solana_program::program::invoke(&ix, &account_infos[..]).unwrap();
        }
    }
    /// serializes the jupiter swap instruction, and swap input data
    pub fn encode_swap_ix_data(&self, swap_inputs: SwapInputs) -> Vec<u8> {
        let mut swap_information = swap_inputs.pack();
        swap_information.insert(0, (*self).into());
        swap_information
    }
    /// serializes the jupiter instruction used to set the token ledger value
    pub fn encode_token_ledger_ix_data(&self) -> Vec<u8> {
        vec![(*self).into()]
    }
    /// create an insruuction which can be used to perform a proxied jupiter
    /// swap via any program implemented the compatible AnyIx instruction parser.
    pub fn encode_swap_ix(
        &self,
        swap_inputs: SwapInputs,
        vaults_program: Pubkey,
        accounts: impl ToAccountMetas,
    ) -> Instruction {
        Instruction {
            program_id: vaults_program,
            accounts: accounts.to_account_metas(None),
            data: self.encode_swap_ix_data(swap_inputs),
        }
    }
    /// create an insruuction which can be used to perform a proxied jupiter
    /// set token ledger ix via any program implemented the compatible AnyIx instruction parser.
    pub fn encode_token_ledger_ix(
        &self,
        vaults_program: Pubkey,
        accounts: impl ToAccountMetas,
    ) -> Instruction {
        Instruction {
            program_id: vaults_program,
            accounts: accounts.to_account_metas(None),
            data: self.encode_token_ledger_ix_data(),
        }
    }
}

pub fn take_accounts_into_metas<'info>(
    accounts: &mut &[AccountInfo<'info>],
    count: usize,
) -> Vec<AccountMeta> {
    let mut account_metas = Vec::with_capacity(count);
    for _ in 0..count {
        let account = &accounts[0];
        account_metas.push(if account.is_writable {
            AccountMeta::new(*account.key, account.is_signer)
        } else {
            AccountMeta::new_readonly(*account.key, account.is_signer)
        });
        // mutate the slice
        *accounts = &accounts[1..];
    }
    account_metas
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_swap_inputs() {
        let input1 = vec![];
        let input2 = SwapInputs {
            input_amount: None,
            min_output: 420_690,
            side: 0,
        }
        .pack();
        let input3 = SwapInputs {
            input_amount: Some(690_420),
            min_output: 69_69,
            side: 0,
        }
        .pack();
        let input4 = SwapInputs {
            input_amount: Some(690_420),
            min_output: 69_69,
            side: 1,
        }
        .pack();
        let input5 = SwapInputs {
            input_amount: None,
            min_output: 69_69,
            side: 1,
        }
        .pack();

        let got_input1 = SwapInputs::new().unpack(&input1[..]);
        let got_input2 = SwapInputs::new().unpack(&input2[..]);
        let got_input3 = SwapInputs::new().unpack(&input3[..]);
        let got_input4 = SwapInputs::new().unpack(&input4[..]);
        let got_input5 = SwapInputs::new().unpack(&input5[..]);

        assert!(got_input1.input_amount.is_none());
        assert!(got_input1.min_output == 0);

        assert!(got_input2.input_amount.is_none());
        assert!(got_input2.min_output == 420_690);

        assert!(got_input3.input_amount.unwrap() == 690_420);
        assert!(got_input3.min_output == 69_69);

        assert!(got_input4.input_amount.unwrap() == 690_420);
        assert!(got_input4.min_output == 69_69);

        assert!(got_input5.input_amount.is_none());
        assert!(got_input5.min_output == 69_69);
        assert!(got_input5.side == 1);
    }
}
