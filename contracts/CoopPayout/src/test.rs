#![cfg(test)]
use soroban_sdk::{Env, Address, BytesN, token, testutils::Address as TestAddress};
use crate::GigEscrow;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::IntoVal;

    fn addr(env: &Env, n: u8) -> Address {
        Address::from_contract_id(&env, &BytesN::from_array(env, &[n;32]))
    }

    // Test 1: Happy path — create escrow, mark delivered, release (token transfer)
    #[test]
    fn happy_path_escrow_release() {
        let env = Env::default();
        env.mock_all_auths();
        let client = addr(&env, 1);
        let freelancer = addr(&env, 2);
        let token_id = BytesN::from_array(&env, &[5u8;32]);

        GigEscrow::initialize(env.clone());

        // Client creates escrow
        env.set_invoker(client.clone());
        let id = GigEscrow::create_escrow(env.clone(), freelancer.clone(), token_id.clone(), 400);

        // Freelancer marks delivered
        env.set_invoker(freelancer.clone());
        GigEscrow::mark_delivered(env.clone(), id);

        // Client releases
        env.set_invoker(client.clone());
        GigEscrow::release(env.clone(), id);

        // Verify escrow state released
        let e = GigEscrow::get_escrow(env.clone(), id);
        assert!(e.released);
    }

    // Test 2: Edge case — unauthorized release attempt
    #[test]
    fn unauthorized_release_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let client = addr(&env, 1);
        let attacker = addr(&env, 9);
        let freelancer = addr(&env, 2);
        let token_id = BytesN::from_array(&env, &[6u8;32]);

        GigEscrow::initialize(env.clone());
        env.set_invoker(client.clone());
        let id = GigEscrow::create_escrow(env.clone(), freelancer.clone(), token_id.clone(), 200);

        // Freelancer marks delivered
        env.set_invoker(freelancer.clone());
        GigEscrow::mark_delivered(env.clone(), id);

        // Attacker tries to release
        env.set_invoker(attacker.clone());
        let res = std::panic::catch_unwind(|| {
            GigEscrow::release(env.clone(), id);
        });
        assert!(res.is_err());
    }

    // Test 3: State verification after create
    #[test]
    fn state_verification_after_create() {
        let env = Env::default();
        env.mock_all_auths();
        let client = addr(&env, 1);
        let freelancer = addr(&env, 2);
        let token_id = BytesN::from_array(&env, &[7u8;32]);

        GigEscrow::initialize(env.clone());
        env.set_invoker(client.clone());
        let id = GigEscrow::create_escrow(env.clone(), freelancer.clone(), token_id.clone(), 1000);

        let e = GigEscrow::get_escrow(env.clone(), id);
        assert_eq!(e.amount, 1000);
        assert_eq!(e.delivered, false);
        assert_eq!(e.released, false);
    }

    // Test 4: Failure when releasing before delivery
    #[test]
    fn release_before_delivery_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let client = addr(&env, 1);
        let freelancer = addr(&env, 2);
        let token_id = BytesN::from_array(&env, &[8u8;32]);

        GigEscrow::initialize(env.clone());
        env.set_invoker(client.clone());
        let id = GigEscrow::create_escrow(env.clone(), freelancer.clone(), token_id.clone(), 50);

        // Client tries to release before freelancer marks delivered
        env.set_invoker(client.clone());
        let res = std::panic::catch_unwind(|| {
            GigEscrow::release(env.clone(), id);
        });
        assert!(res.is_err());
    }

    // Test 5: Duplicate delivery marking fails
    #[test]
    fn duplicate_mark_delivered_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let client = addr(&env, 1);
        let freelancer = addr(&env, 2);
        let token_id = BytesN::from_array(&env, &[9u8;32]);

        GigEscrow::initialize(env.clone());
        env.set_invoker(client.clone());
        let id = GigEscrow::create_escrow(env.clone(), freelancer.clone(), token_id.clone(), 75);

        env.set_invoker(freelancer.clone());
        GigEscrow::mark_delivered(env.clone(), id);

        // Second mark should fail
        let res = std::panic::catch_unwind(|| {
            GigEscrow::mark_delivered(env.clone(), id);
        });
        assert!(res.is_err());
    }
}
