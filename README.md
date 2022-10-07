# juper

Rust jupiter API client with compatible on-chain proxied jupiter swaps via [`AnyIx`](https://github.com/bonedaddy/anyix). Based on [`rust-jup-ag`](https://github.com/mvines/rust-jup-ag).
# crates

## `juper_swap_api`


`juper_swap_api` is a heavily rewritten fork of mvine's `rust-jup-ag` providing both async ahd blocking clients, as well as a simple route cache.

## `juper_swap_cpi`


> **Warning: improper use of this, including the example below will lead to exploitable programs. This must be implemented with care, and developer(s) takes no responsibility for financial loss from any use of this**

a lightweight version of [jupiter-cpi](https://github.com/jup-ag/jupiter-cpi) intended for usage with `AnyIx`. It allows seamless integration of Jupiter's swap api and your programs, enabling things like vault compounding routed through jupiter, with on-chain access controls etc..

### Usage


#### 1) On-Chain Program

In your on-chian program define a function, and an instruction accounts object that looks like  the following.

```rust

#[derive(Accounts)]
pub struct JupiterSwap<'info> {
    /// CHECK: not needed
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    /// CHECK: not needed
    ///
    /// an account that can be used to read various access control settings, etc..
    /// this is intended to be used to restrict access to the `jupiter_swap` function
    pub management: AccountInfo<'info>,
    /// CHECK: not needed
    /// 
    /// an account that is used to read values related to ownership of the token accounts
    /// etc.. this can be used to validate that the token account owners of the jupiter swap
    /// accounts are owned by specific pdas, etc..
    pub vault: AccountInfo<'info>,
    /// CHECK: not needed
    /// 
    /// the actual jupiter program itself
    pub jupiter_program: AccountInfo<'info>,
}

/// all accounts required by this instruction except thos listed in `JupiterSwap`
/// are provided via remaining_accounts, use with caution.
pub fn jupiter_swap<'a, 'b, 'c, 'info>(
    mut ctx: Context<'a, 'b, 'c, 'info, JupiterSwap<'info>>,
    // this must be encoded in the AnyIx format
    input_data: Vec<u8>
) -> Result<()> {
    juper_swap_cpi::process_instructions(
        pda,
        &ctx.accounts.jupiter_program,
        &mut ctx.remaining_accounts.to_owned(),
        &input_data[..],
        None,
    )?;
    Ok(())
}
```

#### 2) Off-Chain Invocation

For example of the off-chain instruction generation see `crates/juper_swap_api`