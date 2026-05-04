#![cfg(test)]
use soroban_sdk::{Env, Address, BytesN, token, testutils::Address as TestAddress, IntoVal};
use crate::CoopPayout;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Symbol, Vec};

    // Helper to create addresses
    fn addr(env: &Env, n: u8) -> Address {
        Address::from_contract_id(&env, &BytesN::from_array(env, &[n;32]))
    }

    // Test 1: Happy path — owner initializes, deposits, allocates, member withdraws (native)
    #[test]
    fn happy_path_withdraw_native() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = addr(&env, 1);
        let member = addr(&env, 2);

        // Initialize
        CoopPayout::initialize(env.clone(), owner.clone());

        // Deposit 1000
        CoopPayout::deposit(env.clone(), 1000);

        // Allocate 300 to member
        CoopPayout::allocate(env.clone(), member.clone(), 300);

        // Member withdraws (native)
        env.set_invoker(member.clone());
        CoopPayout::withdraw(env.clone(), None);

        // Assert allocation is zero and contract balance decreased
        let alloc = CoopPayout::allocation_of(env.clone(), member.clone());
        assert_eq!(alloc, 0);
        let bal = CoopPayout::contract_balance(env.clone());
        assert_eq!(bal, 700);
    }

    // Test 2: Edge case — unauthorized allocate attempt
    #[test]
    fn unauthorized_allocate_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = addr(&env, 1);
        let attacker = addr(&env, 9);
        let member = addr(&env, 2);

        CoopPayout::initialize(env.clone(), owner.clone());
        CoopPayout::deposit(env.clone(), 500);

        // Attacker tries to allocate
        env.set_invoker(attacker.clone());
        let res = std::panic::catch_unwind(|| {
            CoopPayout::allocate(env.clone(), member.clone(), 100);
        });
        assert!(res.is_err());
    }

    // Test 3: State verification after multiple allocations
    #[test]
    fn state_verification_allocations() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = addr(&env, 1);
        let a = addr(&env, 2);
        let b = addr(&env, 3);

        CoopPayout::initialize(env.clone(), owner.clone());
        CoopPayout::deposit(env.clone(), 1000);

        CoopPayout::allocate(env.clone(), a.clone(), 200);
        CoopPayout::allocate(env.clone(), b.clone(), 300);

        let alloc_a = CoopPayout::allocation_of(env.clone(), a.clone());
        let alloc_b = CoopPayout::allocation_of(env.clone(), b.clone());
        assert_eq!(alloc_a, 200);
        assert_eq!(alloc_b, 300);
        let bal = CoopPayout::contract_balance(env.clone());
        assert_eq!(bal, 1000);
    }

    // Test 4: Withdraw token path (uses token client mock)
    #[test]
    fn withdraw_token_transfers() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = addr(&env, 1);
        let member = addr(&env, 2);

        CoopPayout::initialize(env.clone(), owner.clone());

        // Create a mock token contract id
        let token_id = BytesN::from_array(&env, &[7u8;32]);
        // For test, we don't need to mint; token::Client::transfer will be callable in test env

        // Owner allocates token amount
        CoopPayout::allocate(env.clone(), member.clone(), 150);

        // Member withdraws token
        env.set_invoker(member.clone());
        CoopPayout::withdraw(env.clone(), Some(token_id.clone()));

        // Allocation should be zero
        let alloc = CoopPayout::allocation_of(env.clone(), member.clone());
        assert_eq!(alloc, 0);
    }

    // Test 5: Failure when withdrawing with no allocation
    #[test]
    fn withdraw_no_allocation_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let owner = addr(&env, 1);
        let member = addr(&env, 2);

        CoopPayout::initialize(env.clone(), owner.clone());
        env.set_invoker(member.clone());
        let res = std::panic::catch_unwind(|| {
            CoopPayout::withdraw(env.clone(), None);
        });
        assert!(res.is_err());
    }
}
