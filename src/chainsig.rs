use crate::*;
use omni_transaction::signer::types::{SignRequestArgs, mpc_contract};

fn join_all<I>(mut iter: I) -> Promise
where
    I: Iterator<Item = Promise>,
{
    let first = iter.next().expect("Must have at least one promise");
    iter.fold(first, |acc, p| acc.and(p))
}

pub fn internal_request_signatures(
    requests: Vec<SignRequestArgs>,
    mpc_contract_id: AccountId,
) -> Promise {
    // Create promises for all signature requests
    let calls = requests.iter().map(|request| {
        // Validate each request before requesting signature 
        request.validate_payload_length().expect("Invalid payload size");
        
        mpc_contract::ext(mpc_contract_id.clone())
            .with_static_gas(SIGNATURE_GAS)
            .with_attached_deposit(ATTACHED_DEPOSIT)
            .sign(request.clone())
    });

    // Calculate callback gas: constant + (multiplier * number_of_requests)
    let callback_gas = CALLBACK_GAS_CONSTANT.saturating_add(
        CALLBACK_GAS_MULTIPLIER.saturating_mul(requests.len() as u64)
    );
    // Join all promises and then resolve with callback
    join_all(calls).then(
        Contract::ext(env::current_account_id())
            .with_static_gas(callback_gas)
            .resolve_signatures(requests),
    )
}
