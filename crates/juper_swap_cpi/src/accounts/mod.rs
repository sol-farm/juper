use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WhirlpoolSwap<'info> {
    pub swap_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    //#[account(signer)]
    pub token_authority: AccountInfo<'info>,
    #[account(mut)]
    pub whirlpool: AccountInfo<'info>,
    #[account(mut)]
    pub token_owner_account_a: AccountInfo<'info>,
    #[account(mut)]
    pub token_vault_a: AccountInfo<'info>,
    #[account(mut)]
    pub token_owner_account_b: AccountInfo<'info>,
    #[account(mut)]
    pub token_vault_b: AccountInfo<'info>,
    #[account(mut)]
    pub tick_array0: AccountInfo<'info>,
    #[account(mut)]
    pub tick_array1: AccountInfo<'info>,
    #[account(mut)]
    pub tick_array2: AccountInfo<'info>,
    pub oracle: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MercurialExchange<'info> {
    pub swap_program: AccountInfo<'info>,
    pub swap_state: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub pool_authority: AccountInfo<'info>,
    //#[account(signer)]
    pub user_transfer_authority: AccountInfo<'info>,
    #[account(mut)]
    pub source_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub destination_token_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SaberSwap<'info> {
    pub swap_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub swap: AccountInfo<'info>,
    pub swap_authority: AccountInfo<'info>,
    pub user_authority: AccountInfo<'info>,
    #[account(mut)]
    pub input_user_account: AccountInfo<'info>,
    #[account(mut)]
    pub input_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub output_user_account: AccountInfo<'info>,
    #[account(mut)]
    pub output_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub fees_token_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SerumSwap<'info> {
    pub serum_swap_market: SerumSwapMarket<'info>,
    //#[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub order_payer_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    pub dex_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SerumSwapMarket<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    pub vault_signer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetTokenLedger<'info> {
    #[account(mut)]
    pub token_ledger: AccountInfo<'info>,
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RaydiumSwapV2<'info> {
    pub swap_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    #[account(mut)]
    pub amm_id: AccountInfo<'info>,
    pub amm_authority: AccountInfo<'info>,
    #[account(mut)]
    pub amm_open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub pool_coin_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub pool_pc_token_account: AccountInfo<'info>,
    pub serum_program_id: AccountInfo<'info>,
    #[account(mut)]
    pub serum_market: AccountInfo<'info>,
    #[account(mut)]
    pub serum_bids: AccountInfo<'info>,
    #[account(mut)]
    pub serum_asks: AccountInfo<'info>,
    #[account(mut)]
    pub serum_event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub serum_coin_vault_account: AccountInfo<'info>,
    #[account(mut)]
    pub serum_pc_vault_account: AccountInfo<'info>,
    pub serum_vault_signer: AccountInfo<'info>,
    #[account(mut)]
    pub user_source_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_destination_token_account: AccountInfo<'info>,
    //#[account(signer)]
    pub user_source_owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RaydiumSwap<'info> {
    pub swap_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    #[account(mut)]
    pub amm_id: AccountInfo<'info>,
    pub amm_authority: AccountInfo<'info>,
    #[account(mut)]
    pub amm_open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub amm_target_orders: AccountInfo<'info>,
    #[account(mut)]
    pub pool_coin_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub pool_pc_token_account: AccountInfo<'info>,
    pub serum_program_id: AccountInfo<'info>,
    pub serum_market: AccountInfo<'info>,
    #[account(mut)]
    pub serum_bids: AccountInfo<'info>,
    #[account(mut)]
    pub serum_asks: AccountInfo<'info>,
    #[account(mut)]
    pub serum_event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub serum_coin_vault_account: AccountInfo<'info>,
    #[account(mut)]
    pub serum_pc_vault_account: AccountInfo<'info>,
    pub serum_vault_signer: AccountInfo<'info>,
    #[account(mut)]
    pub user_source_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_destination_token_account: AccountInfo<'info>,
    //#[account(signer)]
    pub user_source_owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct LifinityTokenSwap<'info> {
    pub swap_program: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub amm: AccountInfo<'info>,
    //#[account(signer)]
    pub user_transfer_authority: AccountInfo<'info>,
    #[account(mut)]
    pub source_info: AccountInfo<'info>,
    #[account(mut)]
    pub destination_info: AccountInfo<'info>,
    #[account(mut)]
    pub swap_source: AccountInfo<'info>,
    #[account(mut)]
    pub swap_destination: AccountInfo<'info>,
    #[account(mut)]
    pub pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pub fee_account: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub pyth_account: AccountInfo<'info>,
    pub pyth_pc_account: AccountInfo<'info>,
    #[account(mut)]
    pub config_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CykuraSwap<'info> {
    pub swap_program: AccountInfo<'info>,
    //#[account(signer)]
    pub signer: AccountInfo<'info>,
    pub factory_state: AccountInfo<'info>,
    #[account(mut)]
    pub pool_state: AccountInfo<'info>,
    #[account(mut)]
    pub input_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub output_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub input_vault: AccountInfo<'info>,
    #[account(mut)]
    pub output_vault: AccountInfo<'info>,
    #[account(mut)]
    pub last_observation_state: AccountInfo<'info>,
    pub core_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CropperTokenSwap<'info> {
    pub token_swap_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub swap: AccountInfo<'info>,
    pub swap_state: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    //#[account(signer)]
    pub user_transfer_authority: AccountInfo<'info>,
    #[account(mut)]
    pub source: AccountInfo<'info>,
    #[account(mut)]
    pub swap_source: AccountInfo<'info>,
    #[account(mut)]
    pub swap_destination: AccountInfo<'info>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    #[account(mut)]
    pub pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_fee: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AldrinV2Swap<'info> {
    pub swap_program: AccountInfo<'info>,
    pub pool: AccountInfo<'info>,
    pub pool_signer: AccountInfo<'info>,
    #[account(mut)]
    pub pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pub base_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub quote_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub fee_pool_token_account: AccountInfo<'info>,
    //#[account(signer)]
    pub wallet_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_base_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_quote_token_account: AccountInfo<'info>,
    pub curve: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TokenSwap<'info> {
    pub token_swap_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub swap: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    //#[account(signer)]
    pub user_transfer_authority: AccountInfo<'info>,
    #[account(mut)]
    pub source: AccountInfo<'info>,
    #[account(mut)]
    pub swap_source: AccountInfo<'info>,
    #[account(mut)]
    pub swap_destination: AccountInfo<'info>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    #[account(mut)]
    pub pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool_fee: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct RiskCheckAndFee<'info> {
    #[account(mut)]
    pub token_ledger: AccountInfo<'info>,
    #[account(mut)]
    pub user_destination_token_account: AccountInfo<'info>,
    //#[account(signer)]
    pub user_transfer_authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AldrinSwap<'info> {
    pub swap_program: AccountInfo<'info>,
    pub pool: AccountInfo<'info>,
    pub pool_signer: AccountInfo<'info>,
    #[account(mut)]
    pub pool_mint: AccountInfo<'info>,
    #[account(mut)]
    pub base_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub quote_token_vault: AccountInfo<'info>,
    #[account(mut)]
    pub fee_pool_token_account: AccountInfo<'info>,
    //#[account(signer)]
    pub wallet_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_base_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user_quote_token_account: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub struct JupiterSwap {
    /// CHECK: not needed
    //#[account(signer)]
    pub authority: Pubkey,
    /// CHECK: not needed
    ///
    /// an account to read access controlm ifnormation from
    pub management: Pubkey,
    /// CHECK: not needed
    ///
    /// an account to read validation information from
    pub vault: Pubkey,
    /// CHECK: not needed
    pub jupiter_program: Pubkey,
}

impl ToAccountMetas for JupiterSwap {
    fn to_account_metas(&self, _is_signer: Option<bool>) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(self.authority, true),
            AccountMeta::new_readonly(self.management, false),
            AccountMeta::new_readonly(self.vault, false),
            AccountMeta::new_readonly(self.jupiter_program, false),
        ]
    }
}
