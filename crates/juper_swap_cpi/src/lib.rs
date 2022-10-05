pub mod accounts;
pub mod instructions;

use solana_program::{self, pubkey::Pubkey};
use static_pubkey::static_pubkey;

use instructions::Side;

/// the program id of the v3 jupiter aggregator
pub const JUPITER_V3_AGG_ID: Pubkey = static_pubkey!("JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph");

/// alias to satisfy anchor codegen requirements
pub const ID: Pubkey = JUPITER_V3_AGG_ID;

use anchor_lang::{prelude::*, InstructionData};
use anyix;
use solana_program::{instruction::Instruction, program_pack::Pack};
use std::collections::BTreeMap;

//#[inline(always)]
pub fn process_instructions<'info>(
    // the account address that should own the token accounts which will receive
    // the outputs and inputs of the swaps
    wanted_token_owner: Pubkey,
    jupiter_program_account: &AccountInfo<'info>,
    remaining_accounts: &mut [AccountInfo<'info>],
    data: &[u8],
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    assert!(jupiter_program_account.key.eq(&JUPITER_V3_AGG_ID));

    let any_ix = anyix::AnyIx::unpack(data)?;
    let anyix::AnyIx {
        num_instructions,
        instruction_data_sizes: _,
        instruction_datas,
        instruction_account_counts,
    } = any_ix;
    let mut offset = 0;
    for idx in 0..num_instructions {
        msg!("processing ix {}", idx);
        let accounts = &remaining_accounts[offset as usize..];
        //offset += accounts.len();
        offset += instruction_account_counts[idx as usize] as usize;

        // the first element of the data slice is going to be the JupiterIx variant
        let jupiter_ix: JupiterIx = From::from(instruction_datas[idx as usize][0]);
        msg!("unpacking swap inputs");
        let swap_inputs = SwapInputs::new().unpack(&instruction_datas[idx as usize][1..]);
        msg!("validating");
        //assert!(jupiter_ix.validate(&accounts[..], wanted_token_owner));
        msg!("executing");
        if idx == 0 {
            continue;
        }
        jupiter_ix.execute(
            accounts,
            seeds,
            swap_inputs.side(),
            swap_inputs.input_amount,
            swap_inputs.min_output,
            wanted_token_owner,
        );
    }
    Ok(())
}

/// given instruction data, process it to determine what if any
/// jupiter instructions are being invoked, and the input arguments
pub fn process_jupiter_instruction(input: &[u8]) -> Result<(JupiterIx, SwapInputs)> {
    if input.len() > 8 {
        let (ix_data, inputs) = input.split_at(8);
        let jupiter_ix: JupiterIx = TryFrom::try_from(ix_data)?;
        return Ok((jupiter_ix, jupiter_ix.get_swap_inputs(inputs)?));
    } else {
        let jupiter_ix: JupiterIx = TryFrom::try_from(input)?;
        return Ok((
            jupiter_ix,
            SwapInputs {
                input_amount: None,
                min_output: 0,
                side: 0,
            },
        ));
    }
}

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
    pub fn unpack(&mut self, data: &[u8]) -> Self {
        let data_len = data.len();
        if data_len == 17 {
            let (input_amount, rest) = data.split_at(8);
            let (min_output, rest) = rest.split_at(16);
            self.input_amount = Some(u64::try_from_slice(input_amount).unwrap());
            self.min_output = u64::try_from_slice(min_output).unwrap();
            self.side = rest[0];
        } else if data_len == 16 {
            let (input_amount, min_output) = data.split_at(8);
            self.input_amount = Some(u64::try_from_slice(input_amount).unwrap());
            self.min_output = u64::try_from_slice(min_output).unwrap();
        } else if data_len == 9 {
            self.min_output = u64::try_from_slice(&data[0..8]).unwrap();
            self.side = data[8];
        } else if data_len == 8 {
            self.min_output = u64::try_from_slice(data).unwrap();
        }
        *self
    }
    pub fn pack(self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(16);
        if let Some(input_amount) = self.input_amount {
            buffer.extend_from_slice(&input_amount.to_le_bytes()[..]);
        }
        buffer.extend_from_slice(&self.min_output.to_le_bytes()[..]);
        buffer
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
        }
    }
}

/// given what is presumed to be anchor program instruction data
/// which is equal to or greater than  8 bytes, determine which
/// of the JupiterIx variants this instruction data is for.
impl TryFrom<&[u8]> for JupiterIx {
    type Error = ProgramError;
    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        use instructions::sighashes::*;
        if value.len() < 8 {
            return Err(ProgramError::AccountDataTooSmall);
        }
        let ix_data = &value[0..8];
        if ix_data.eq(&MERCURIAL_EXCHANGE) {
            return Ok(Self::MercurialExchange);
        } else if ix_data.eq(&SABER_SWAP) {
            return Ok(Self::Saber);
        } else if ix_data.eq(&SERUM_SWAP) {
            return Ok(Self::Serum);
        } else if ix_data.eq(&TOKEN_SWAP) {
            return Ok(Self::TokenSwap);
        } else if ix_data.eq(&STEP_TOKEN_SWAP) {
            msg!("step unsupported");
            return Err(ProgramError::InvalidInstructionData);
        } else if ix_data.eq(&CROPPER_TOKEN_SWAP) {
            return Ok(Self::CropperTokenSwap);
        } else if ix_data.eq(&RAYDIUM_SWAP) {
            return Ok(Self::RaydiumSwap);
        } else if ix_data.eq(&RAYDIUM_SWAP_V2) {
            return Ok(Self::RaydiumSwapV2);
        } else if ix_data.eq(&CREMA_TOKEN_SWAP) {
            msg!("crema unsupported");
            return Err(ProgramError::InvalidInstructionData);
        } else if ix_data.eq(&LIFINITY_TOKEN_SWAP) {
            return Ok(Self::LifinityTokenSwap);
        } else if ix_data.eq(&CYKURA_SWAP) {
            return Ok(Self::CykuraTokenSwap);
        } else if ix_data.eq(&WHIRLPOOL_SWAP) {
            return Ok(Self::Whirlpool);
        } else if ix_data.eq(&SET_TOKEN_LEDGER) {
            return Ok(Self::SetTokenLedger);
        } else {
            return Err(ProgramError::InvalidInstructionData);
        }
    }
}

impl JupiterIx {
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
                    return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
                    }
                }
            }
            JupiterIx::CropperTokenSwap => {
                match instructions::cropper::CropperTokenSwap::try_from_slice(data) {
                    Ok(input) => Ok(SwapInputs {
                        input_amount: input._in_amount,
                        min_output: input._minimum_out_amount,
                        side: 0,
                    }),
                    Err(err) => {
                        msg!("failed to parse cropper swap {:#?}", err);
                        return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
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
                        return Err(ProgramError::InvalidInstructionData.into());
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
                    return Err(ProgramError::InvalidInstructionData.into());
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
                    return Err(ProgramError::InvalidInstructionData.into());
                }
            },
            JupiterIx::SetTokenLedger => Ok(Default::default()),
        }
    }
    pub fn execute<'info>(
        &self,
        mut accounts: &[AccountInfo<'info>],
        seeds: Option<&[&[&[u8]]]>,
        side: Side,
        input: Option<u64>,
        min_output: u64,
        signer: Pubkey,
    ) {
        let (mut ix, account_infos, skip_signer) = match self {
            Self::TokenSwap => {
                let mer_swap = accounts::TokenSwap::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let ix_data = instructions::token_swap::TokenSwap {
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let source_token_account =
                    spl_token::state::Account::unpack(&mer_swap.source.data.borrow()).unwrap();
                let dest_token_account =
                    spl_token::state::Account::unpack(&mer_swap.destination.data.borrow()).unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::AldrinV2Swap => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                    _side: side,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::CropperTokenSwap => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::CykuraTokenSwap => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::LifinityTokenSwap => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::MercurialExchange => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: mer_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, mer_swap.to_account_infos(), false)
            }
            Self::RaydiumSwap => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: ray_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, ray_swap.to_account_infos(), false)
            }
            Self::RaydiumSwapV2 => {
                let ray_swap = accounts::RaydiumSwapV2::try_accounts(
                    &JUPITER_V3_AGG_ID,
                    &mut accounts,
                    &[],
                    &mut BTreeMap::default(),
                )
                .unwrap();
                let ix_data = instructions::raydium_v2::RaydiumSwapV2 {
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
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
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: ray_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, ray_swap.to_account_infos(), false)
            }
            Self::Whirlpool => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _a_to_b: side.a_to_b(),
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: whirlpool_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, whirlpool_swap.to_account_infos(), false)
            }
            Self::Serum => {
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

                let source_token_account =
                    spl_token::state::Account::unpack(&serum_swap.coin_wallet.data.borrow())
                        .unwrap();
                let dest_token_account =
                    spl_token::state::Account::unpack(&serum_swap.pc_wallet.data.borrow()).unwrap();
                assert!(source_token_account.owner.eq(&signer));
                assert!(dest_token_account.owner.eq(&signer));
                let ix_data = instructions::serum::SerumSwap {
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _side: side,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: serum_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, serum_swap.to_account_infos(), false)
            }
            Self::SetTokenLedger => {
                let mut token_ledger = accounts::SetTokenLedger::try_accounts(
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
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: token_ledger.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, token_ledger.to_account_infos(), true)
            }
            Self::Saber => {
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
                    _in_amount: input,
                    _minimum_out_amount: min_output,
                    _platform_fee_bps: 0,
                }
                .data();
                let mut ix = Instruction {
                    program_id: JUPITER_V3_AGG_ID,
                    accounts: saber_swap.to_account_metas(Some(true)),
                    data: ix_data,
                };
                (ix, saber_swap.to_account_infos(), false)
            }
        };

        ix.accounts.iter_mut().for_each(|acct| {
            // we need to do this because the encoded transactions
            // from anyix will override the signer field
            if acct.pubkey.eq(&signer) {
                acct.is_signer = true;
            }
        });
        if !skip_signer {
            if let Some(seeds) = seeds {
                anchor_lang::solana_program::program::invoke_signed(&ix, &account_infos[..], seeds)
                    .unwrap();
            }
        } else {
            anchor_lang::solana_program::program::invoke(&ix, &account_infos[..]).unwrap();
        }
    }
    ///  given the swap_input, encode the JupiterIx object into AnyIX instruction data
    pub fn encode_swap_ix_data(&self, swap_inputs: SwapInputs) -> Vec<u8> {
        let mut swap_information = swap_inputs.pack();
        swap_information.insert(0, (*self).into());
        swap_information
    }
    pub fn encode_token_ledger_ix_data(&self) -> Vec<u8> {
        vec![(*self).into()]
    }
    /// encodes given JupiterIx into swap instruction data suitable for parsing
    /// by any
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
    /// encodes the goven JupiterIx into token ledger instruction data
    /// suitable for parsing by AnyIx
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
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_swap_inputs_empty() {
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

        let got_input1 = SwapInputs::new().unpack(&input1[..]);
        let got_input2 = SwapInputs::new().unpack(&input2[..]);
        let got_input3 = SwapInputs::new().unpack(&input3[..]);

        assert!(got_input1.input_amount.is_none());
        assert!(got_input1.min_output == 0);

        assert!(got_input2.input_amount.is_none());
        assert!(got_input2.min_output == 420_690);

        assert!(got_input3.input_amount.unwrap() == 690_420);
        assert!(got_input3.min_output == 69_69);
    }
}
