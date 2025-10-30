# multiple-signatures

This contract takes an array of signature requests, gets signatures for them and returns them as an array. This is useful when needing to make multiple signature requests from Aurora as an Aurora to NEAR cross contract call is very gas expensive, thus it is more gas efficient to batch the signature requests on the NEAR side.

## Notice 

Before using this repo please review the [NOTICE.txt](./NOTICE.txt) file

## One Contract per Protocol

Since the accounts that can be signed for are fixed for a given predecessor caller to the MPC, the accounts are tied to this contract. To maintain the same accounts you need to maintain this contract under the same NEAR account name. The contract must only let your contract on Aurora call it, otherwise anyone would be able to sign for the derived accounts.

## Deploying 

Download cargo near 

https://github.com/near/cargo-near

Create account

```bash
cargo near create-dev-account
```

Deploy

```bash
cargo near deploy
```

Init args 

```json
{
    "mpc_contract_id": "", // v1.signer-prod.testnet for testnet v1.signer for mainnet
    "owner_id": "", // Can set new owners, new permitted caller, update the contract code - should likely be a multisig 
    "permitted_caller": "", // The account or contract that is permitted to call the request_signatures function
}
```


## Gas 

The contract was tested for gas consumed for a different numbers of requests

1 sig = 20 tgas
6 sig = 110 tgas 
10 sig = 185 tgas
15 sig = 290 tgas 

There are five parts where gas is consumed 

pre request constant = c1
callback constant = c2 
pre request multiplier = x
post request multiplier = y
gas per mpc call = 15 tgas 

This means the contract must have gas requirements that satisfying the following inequalities (this includes a 10 tgas buffer).

x+y+c1+c2>15
6x+6y+c1+c2>30
10x+10y+c1+c2>45
15x+15y+c1+c2>75

Reasonable values that minimize these are 

x=2 
y=2
c1=8
c2=8

The required gas goes as 

tgas required = n(15+x+y)+c1+c2

where n is the number of requests 

For these chosen values 

tgas required = 19n+16

Gas may be able to be optimized further with further testing

## Updating the contract 

See https://docs.near.org/smart-contracts/release/upgrade#programmatic-update