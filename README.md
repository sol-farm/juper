# juper

opinionated rewrite of `mvines/rust-jup-ag`.

# crates

## `swap_api`


`juper` is a heavily rewritten fork of mvine's `rust-jup-ag` providing both async ahd blocking clients, as well as a minimalistic route cache. support for processing swap api encoded transactions into the AnyIx format is included as well.

## `swap_cpi`

`swap_cpi` is designed to leverage the output from jupiter's swap api, and relay it through an on-chain program, minimizing the transaction size by 5 bytes per instruction. This means a 3 legged swap will save 15 bytes!


# note

if you make any money with these modifications, pls donate to your local food bank, money is like cow manure, it works better when spread around.