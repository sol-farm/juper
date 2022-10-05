//! this module contains the instruction sighashes for various jupiter v3 aggregator functions
pub const MERCURIAL_EXCHANGE: [u8; 8] = [31, 248, 60, 226, 215, 168, 55, 199];
pub const SABER_SWAP: [u8; 8] = [64, 62, 98, 226, 52, 74, 37, 178];
pub const SERUM_SWAP: [u8; 8] = [88, 183, 70, 249, 214, 118, 82, 210];
pub const TOKEN_SWAP: [u8; 8] = [187, 192, 118, 212, 62, 109, 28, 213];
pub const STEP_TOKEN_SWAP: [u8; 8] = [55, 100, 17, 243, 242, 181, 43, 165];
pub const CROPPER_TOKEN_SWAP: [u8; 8] = [167, 38, 59, 37, 132, 60, 95, 68];
pub const RAYDIUM_SWAP: [u8; 8] = [177, 173, 42, 240, 184, 4, 124, 81];
pub const RAYDIUM_SWAP_V2: [u8; 8] = [69, 227, 98, 93, 237, 202, 223, 140];
pub const ALDRIN_SWAP: [u8; 8] = [251, 232, 119, 166, 225, 185, 169, 161];
pub const ALDRIN_V2_SWAP: [u8; 8] = [190, 166, 89, 139, 33, 152, 16, 10];
pub const CREMA_TOKEN_SWAP: [u8; 8] = [235, 160, 175, 122, 61, 177, 2, 247];
pub const LIFINITY_TOKEN_SWAP: [u8; 8] = [0, 49, 246, 1, 36, 153, 11, 93];
pub const CYKURA_SWAP: [u8; 8] = [38, 241, 21, 107, 120, 59, 184, 249];
pub const WHIRLPOOL_SWAP: [u8; 8] = [123, 229, 184, 63, 12, 0, 92, 145];
pub const SET_TOKEN_LEDGER: [u8; 8] = [228, 85, 185, 112, 78, 79, 77, 2];

/// the 8 byte instruction sighash for `jupiter_swap` which is expected ot be in any program
/// that supports AnyIx
pub const JUPITER_SWAP_SIGHASH: [u8; 8] = [116, 207, 0, 196, 252, 120, 243, 18];
