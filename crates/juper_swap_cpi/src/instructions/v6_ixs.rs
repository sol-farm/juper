use solana_program::{account_info::AccountInfo, instruction::AccountMeta, program_pack::Pack, pubkey::Pubkey};

/// sighash of the `route` ix
pub const ROUTE_SIGHASH: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
/// sighash of the `route_with_token_ledger` ix
pub const ROUTE_WITH_TOKEN_LEDGER_SIGHASH: [u8; 8] = [150, 86, 71, 116, 167, 93, 14, 104];
/// sighash of the `shared_accounts_route` ix
pub const SHARED_ACCOUNTS_ROUTE_SIGHASH: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];
/// sighash of the `shared_accounts_route_with_token_ledger`
pub const SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER_SIGHASH: [u8; 8] =
    [230, 121, 143, 80, 119, 159, 106, 170];
/// sighash of the `shared_accounts_exact_out_route`
pub const SHARED_ACCOUNTS_EXACT_OUT_ROUTE_SIGHASH: [u8; 8] = [176, 209, 105, 168, 154, 125, 69, 62];

pub const SPL_TOKEN_2022_ID: Pubkey = static_pubkey::static_pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

#[derive(Clone, Copy)]
pub enum V6Instructions {
    Route,
    RouteWithTokenLedger,
    SharedAccountsRoute,
    SharedAccountsRouteWithTokenLedger,
    SharedAccountsExactOut,
}

impl TryFrom<&[u8; 8]> for V6Instructions {
    type Error = String;
    fn try_from(value: &[u8; 8]) -> Result<Self, Self::Error> {
        if value.len() != 8 {
            return Err("invalid_data_length".to_string());
        }
        if ROUTE_SIGHASH.eq(value) {
            Ok(Self::Route)
        } else if ROUTE_WITH_TOKEN_LEDGER_SIGHASH.eq(value) {
            Ok(Self::RouteWithTokenLedger)
        } else if SHARED_ACCOUNTS_ROUTE_SIGHASH.eq(value) {
            Ok(Self::SharedAccountsRoute)
        } else if SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER_SIGHASH.eq(value) {
            Ok(Self::SharedAccountsRouteWithTokenLedger)
        } else if SHARED_ACCOUNTS_EXACT_OUT_ROUTE_SIGHASH.eq(value) {
            Ok(Self::SharedAccountsRouteWithTokenLedger)
        } else {
            Err("invalid_ix".to_string())
        }
    }
}

/// expected account owners
pub struct V6InstructionVerificationOpts {
    pub token_program: Pubkey,
}

impl V6Instructions {
    pub fn validate_accounts(
        self,
        remaining_accounts: &[AccountInfo],
        wanted_transfer_authority: Pubkey,
        want_user_source_token_account_owner: Pubkey,
        wanted_user_destination_token_account_owner: Pubkey,
    ) {
        match self {
            Self::Route => {
                // tokenProgram [0]
                // userTransferAuthority  [1]
                // userSourceTokenAccount [2] 
                // userDestinationTokenAccount [3]
                // destinationTokenAccount [4]
                // destinationMint [5]
                // platformFeeAccount [6]
                // eventAuthority [7]
                // program [8] - todo: verify
                assert_eq!(*remaining_accounts[0].key, spl_token::id());
                assert_eq!(*remaining_accounts[1].key, wanted_transfer_authority);
                self.validate_token_account_owner(
                    want_user_source_token_account_owner, 
                    &remaining_accounts[2],
                );
                self.validate_token_account_owner(
                    wanted_user_destination_token_account_owner, 
                    &remaining_accounts[3],
                );

            }
            Self::RouteWithTokenLedger => {
                // tokenProgram [0]
                // userTransferAuthority [1]
                // userSourceTokenAccount [2]
                // userDestinationTokenAccount [3]
                // destinationTokenAccount [4]
                // destinationMint [5]
                // platformFeeAccount [6]
                // tokenLedger [7] - todo: verify
                // eventAuthority [8]
                // program [9] - todo: verify
                assert_eq!(*remaining_accounts[0].key, spl_token::id());
                assert_eq!(*remaining_accounts[1].key, wanted_transfer_authority);
                self.validate_token_account_owner(
                    want_user_source_token_account_owner, 
                    &remaining_accounts[2],
                );
                self.validate_token_account_owner(
                    wanted_user_destination_token_account_owner, 
                    &remaining_accounts[3],
                );
            }
            Self::SharedAccountsRoute => {
                // tokenProgram [0]
                // programAuthority [1]
                // userTransferAuthority [2]
                // sourceTokenAccount [3]
                // programSourceTokenAccount [4]
                // programDestinationTokenAccount [5]
                // destinationTokenAccount [6]
                // sourceMint [7]
                // destinationMint [8]
                // platformFeeAccount [9]
                // token2022Program [10] - todo: verify
                // eventAuthority [11] - todo: verify
                // program [12] - todo: verify
                assert_eq!(*remaining_accounts[0].key, spl_token::id());
                assert_eq!(*remaining_accounts[2].key, wanted_transfer_authority);
                self.validate_token_account_owner(
                    want_user_source_token_account_owner, 
                    &remaining_accounts[3],
                );
                self.validate_token_account_owner(
                    wanted_user_destination_token_account_owner, 
                    &remaining_accounts[4],
                );
                assert_eq!(*remaining_accounts[10].key, SPL_TOKEN_2022_ID);
            }
            Self::SharedAccountsRouteWithTokenLedger => {
                // tokenProgram [0]
                // programAuthority [1]
                // userTransferAuthority [2]
                // sourceTokenAccount [3]
                // programSourceTokenAccount [3]
                // programDestinationTokenAccount [4]
                // destinationTokenAccount [5]
                // sourceMint [6]
                // destinationMint [7]
                // platformFeeAccount [8]
                // token2022Program [9]
                // tokenLedger [10]
                // eventAuthority [11]
                // program [12]- todo: verify
                assert_eq!(*remaining_accounts[0].key, spl_token::id());
                assert_eq!(*remaining_accounts[2].key, wanted_transfer_authority);
                self.validate_token_account_owner(
                    want_user_source_token_account_owner, 
                    &remaining_accounts[3],
                );
                self.validate_token_account_owner(
                    wanted_user_destination_token_account_owner, 
                    &remaining_accounts[3],
                );
                assert_eq!(*remaining_accounts[8].key, SPL_TOKEN_2022_ID);                
                    
            }
            Self::SharedAccountsExactOut => {
                // tokenProgram [0]
                // programAuthority [1]
                // userTransferAuthority [2]
                // sourceTokenAccount [3]
                // programSourceTokenAccount [4]
                // programDestinationTokenAccount [5]
                // destinationTokenAccount [6]
                // sourceMint [7]
                // destinationMint [8]
                // platformFeeAccount [9]
                // token2022Program [10]
                // eventAuthority [11]
                // program [12]
                assert_eq!(*remaining_accounts[0].key, spl_token::id());
                assert_eq!(*remaining_accounts[2].key, wanted_transfer_authority);
                self.validate_token_account_owner(
                    want_user_source_token_account_owner, 
                    &remaining_accounts[3],
                );
                self.validate_token_account_owner(
                    wanted_user_destination_token_account_owner, 
                    &remaining_accounts[6],
                );
                assert_eq!(*remaining_accounts[10].key, SPL_TOKEN_2022_ID);                                   
            }
        }
    }

fn validate_token_account_owner(
    self,
    want_owner: Pubkey,
    account: &AccountInfo
) {
    let token_account = spl_token::state::Account::unpack(&account.data.borrow()).unwrap();
    assert_eq!(token_account.owner, want_owner);
}
}
