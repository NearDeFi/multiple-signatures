use crate::*;

#[near]
impl Contract {
    pub fn update_contract(&self) -> Promise {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "Only the owner can update the code"
        );

        // Receive the code directly from the input to avoid the
        // GAS overhead of deserializing parameters
        let code = env::input().expect("Error: No input").to_vec();

        Promise::new(env::current_account_id())
            .deploy_contract(code)
            .as_return()
    }
}
