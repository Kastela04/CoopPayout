#![no_std]
use soroban_sdk::{contractimpl, contracttype, Address, Env, Symbol, Vec, Map, BytesN, IntoVal, token, panic_with_error};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Owner,
    Allocations, // Map<Address, i128> stored as Map
    Balance,     // i128 representing contract-held XLM (simulated)
}

/// Simple CoopPayout contract:
/// - Owner sets allocations for recipients
/// - Recipients call withdraw to receive allocated amount
/// - Contract uses token::Client to transfer native XLM or USDC (token address passed in withdraw)
pub struct CoopPayout;

#[contractimpl]
impl CoopPayout {
    // Initialize contract owner
    pub fn initialize(env: Env, owner: Address) {
        if env.storage().has(&DataKey::Owner) {
            panic_with_error!(&env, "already_initialized");
        }
        env.storage().set(&DataKey::Owner, &owner);
        let m: Map<Address, i128> = Map::new(&env);
        env.storage().set(&DataKey::Allocations, &m);
        env.storage().set(&DataKey::Balance, &0i128);
    }

    // Owner records a deposit (for demo/test we simulate XLM deposit by calling this)
    // In production, deposit would be an actual token transfer to contract address.
    pub fn deposit(env: Env, amount: i128) {
        let caller = env.invoker();
        let owner: Address = env.storage().get_unchecked(&DataKey::Owner).unwrap();
        if caller != owner {
            panic_with_error!(&env, "unauthorized");
        }
        let mut bal: i128 = env.storage().get_unchecked(&DataKey::Balance).unwrap();
        bal += amount;
        env.storage().set(&DataKey::Balance, &bal);
    }

    // Owner allocates amount to a recipient (adds to their allocation)
    pub fn allocate(env: Env, recipient: Address, amount: i128) {
        let caller = env.invoker();
        let owner: Address = env.storage().get_unchecked(&DataKey::Owner).unwrap();
        if caller != owner {
            panic_with_error!(&env, "unauthorized");
        }
        let mut allocs: Map<Address, i128> = env.storage().get_unchecked(&DataKey::Allocations).unwrap();
        let prev = allocs.get(recipient.clone()).unwrap_or(0i128);
        allocs.set(recipient.clone(), prev + amount);
        env.storage().set(&DataKey::Allocations, &allocs);
    }

    // Member withdraws their allocation. token_id = None for native XLM, or Some(token_contract_id) for USDC.
    pub fn withdraw(env: Env, token_id: Option<BytesN<32>>) {
        let caller = env.invoker();
        let mut allocs: Map<Address, i128> = env.storage().get_unchecked(&DataKey::Allocations).unwrap();
        let amt = allocs.get(caller.clone()).unwrap_or(0i128);
        if amt <= 0 {
            panic_with_error!(&env, "no_allocation");
        }
        // Reduce allocation to zero
        allocs.set(caller.clone(), 0i128);
        env.storage().set(&DataKey::Allocations, &allocs);

        // Reduce contract balance (simulated) if native
        if token_id.is_none() {
            let mut bal: i128 = env.storage().get_unchecked(&DataKey::Balance).unwrap();
            if bal < amt {
                panic_with_error!(&env, "insufficient_contract_balance");
            }
            bal -= amt;
            env.storage().set(&DataKey::Balance, &bal);
            // In real deployment, contract would call token::Client::transfer for native asset
            // Here we emit an event to indicate transfer (tests will assert state)
            env.events().publish((Symbol::short("withdraw_native"),), (caller, amt));
        } else {
            // For token transfers, call token client
            let token_contract = token::Client::new(&env, &token_id.unwrap());
            // transfer from contract (self) to caller
            token_contract.transfer(&env.current_contract_address(), &caller, &amt);
            env.events().publish((Symbol::short("withdraw_token"),), (caller, amt));
        }
    }

    // View allocation for an address
    pub fn allocation_of(env: Env, who: Address) -> i128 {
        let allocs: Map<Address, i128> = env.storage().get_unchecked(&DataKey::Allocations).unwrap();
        allocs.get(who).unwrap_or(0i128)
    }

    // View contract balance (simulated)
    pub fn contract_balance(env: Env) -> i128 {
        env.storage().get_unchecked(&DataKey::Balance).unwrap()
    }
}
