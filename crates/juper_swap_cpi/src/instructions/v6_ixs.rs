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
/// all whitelisted sighashes for the v6 aggregator
pub const V6_AGG_SIGHASHES: [[u8; 8]; 5] = [
    ROUTE_SIGHASH,
    ROUTE_WITH_TOKEN_LEDGER_SIGHASH,
    SHARED_ACCOUNTS_ROUTE_SIGHASH,
    SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER_SIGHASH,
    SHARED_ACCOUNTS_EXACT_OUT_ROUTE_SIGHASH,
];
