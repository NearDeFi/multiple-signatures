use near_sdk::{
    env::{self},
    log, near, AccountId, Gas, NearToken, PanicOnDefault, Promise, 
    require,
};
use omni_transaction::signer::types::{SignRequestArgs, SignatureResponse};

mod chainsig;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub mpc_contract_id: AccountId,
    pub owner_id: AccountId,
    pub permitted_caller: AccountId,
}

// Gas can be optimized by dynamically changing the gas required for initial gas and callback gas based on the number of requests
const SIGNATURE_GAS: Gas = Gas::from_tgas(15);
const INITIAL_GAS_CONSTANT: Gas = Gas::from_tgas(8);
const CALLBACK_GAS_CONSTANT: Gas = Gas::from_tgas(8);
const INITIAL_GAS_MULTIPLIER: Gas = Gas::from_tgas(2);
const CALLBACK_GAS_MULTIPLIER: Gas = Gas::from_tgas(2);

const ATTACHED_DEPOSIT: NearToken = NearToken::from_yoctonear(1);

#[near]
impl Contract {
    #[init]
    pub fn new(mpc_contract_id: AccountId, owner_id: AccountId, permitted_caller: AccountId) -> Self {
        Self {
            mpc_contract_id, // v1.signer-prod.testnet for testnet v1.signer for mainnet
            permitted_caller,
            owner_id,
        }
    }

    pub fn update_permitted_caller(&mut self, new_permitted_caller: AccountId) {
        require!(env::predecessor_account_id() == self.owner_id, "Only the owner can call this contract");
        self.permitted_caller = new_permitted_caller;
    }

    pub fn update_owner(&mut self, new_owner_id: AccountId) {
        require!(env::predecessor_account_id() == self.owner_id, "Only the owner can call this contract");
        self.owner_id = new_owner_id;
    }

    pub fn request_signatures(&mut self, requests: Vec<SignRequestArgs>) -> Promise {
        require!(env::predecessor_account_id() == self.permitted_caller, "Only the permitted caller can call this contract");
        log!("Requesting signatures for {} requests", requests.len());
        require_enough_gas(requests.len() as u64);
        chainsig::internal_request_signatures(requests, self.mpc_contract_id.clone())
    }

    #[private]
    pub fn resolve_signatures(
        &self,
        requests: Vec<SignRequestArgs>,
    ) -> Vec<(SignRequestArgs, Result<SignatureResponse, ()>)> {
        let mut results = Vec::new();
        let mut successful_count = 0;

        for (i, request) in requests.into_iter().enumerate() {
            let response = match env::promise_result(i as u64) {
                near_sdk::PromiseResult::Successful(data) => {
                    // Deserialize the SignatureResponse
                    match serde_json::from_slice::<SignatureResponse>(&data) {
                        Ok(sig_response) => {
                            successful_count += 1;
                            Ok(sig_response)
                        }
                        Err(e) => {
                            log!("Failed to deserialize signature response for request {}: {:?}", i, e);
                            Err(())
                        }
                    }
                }
                _ => {
                    log!("Signature request {} failed", i);
                    Err(())
                }
            };

            results.push((request, response));
        }

        let failed_count = results.len() - successful_count;
        log!(
            "Resolved {} signature results: {} successful, {} failed",
            results.len(),
            successful_count,
            failed_count
        );
        results
    }
}

fn require_enough_gas(number_of_requests: u64) {
    log!("Prepaid gas: {}", env::prepaid_gas());
    
    // Calculate initial gas: constant + (multiplier * number_of_requests)
    let initial_gas = INITIAL_GAS_CONSTANT.saturating_add(
        INITIAL_GAS_MULTIPLIER.saturating_mul(number_of_requests)
    );
    
    // Calculate callback gas: constant + (multiplier * number_of_requests)
    let callback_gas = CALLBACK_GAS_CONSTANT.saturating_add(
        CALLBACK_GAS_MULTIPLIER.saturating_mul(number_of_requests)
    );
    
    // Calculate total required gas: (signature gas * number_of_requests) + initial gas + callback gas
    let required_gas = SIGNATURE_GAS.saturating_mul(number_of_requests)
        .saturating_add(initial_gas)
        .saturating_add(callback_gas);
    
    log!("Required gas: {} (signatures: {}, initial: {}, callback: {})", 
         required_gas, 
         SIGNATURE_GAS.saturating_mul(number_of_requests),
         initial_gas, 
         callback_gas);
    require!(env::prepaid_gas() >= required_gas, "Insufficient gas");
}