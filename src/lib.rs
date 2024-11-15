use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault, PromiseOrValue,
};
use near_sdk::collections::{LookupMap};

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct EmissionsAccount {
    pub initial_emissions: u64,
    pub decay_factor: f64,
    pub current_month: u32,
    pub current_emissions: u64,
    pub last_mint_timestamp: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct RafflePool {
    pub pool_id: u32,
    pub amount: u128,
    pub total_amount: u128,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TappingPool {
    pub pool_id: u32,
    pub amount: u128,
}


#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    emissions_account: LookupMap<AccountId, EmissionsAccount>,
    loot_raffle_pool: LookupMap<u32, RafflePool>,
    global_tapping_pool: LookupMap<u32, TappingPool>,

}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='https://red-defensive-termite-556.mypinata.cloud/ipfs/QmUCUAABBsqkhSw3HoeMtecwVAeKBmxUgj2GLwmxuNojbV' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        require!(!env::state_exists(), "Already initialized");
        
        let mut this = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "PUSH THE BUTTON PTB".to_string(),
                symbol: "PUSH".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 5,
            })),
            emissions_account: LookupMap::new(b"e"),
            loot_raffle_pool: LookupMap::new(b"l"),
            global_tapping_pool: LookupMap::new(b"g"),
        };

        // Initialize Emissions Account
        this.emissions_account.insert(&owner_id, &EmissionsAccount {
            initial_emissions: 3_000_000_000,
            decay_factor: 0.8705505633,
            current_month: 0,
            current_emissions: 3_000_000_000,
            last_mint_timestamp: env::block_timestamp(),
        });

        // Initialize Raffle and Tapping Pools
        this.loot_raffle_pool.insert(&1, &RafflePool {
            pool_id: 1,
            amount: 50_000_000_00000,
            total_amount: 0,
        });
        this.global_tapping_pool.insert(&2, &TappingPool {
            pool_id: 2,
            amount: 1_000_000_000_00000,
        });

        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());

        this
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            emissions_account: LookupMap::new(b"e"),
            loot_raffle_pool: LookupMap::new(b"l"),
            global_tapping_pool: LookupMap::new(b"g"),

        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());

        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: total_supply,
            memo: Some("new tokens are minted"),
        }
        .emit();

        this
    }

    // Mint function added here
    pub fn mint(&mut self, owner_id: AccountId, amount: U128) {
        self.token.internal_deposit(&owner_id, amount.into());
    
        near_contract_standards::fungible_token::events::FtMint {
        owner_id: &owner_id,
        amount: amount,
        memo: Some("tokens minted"),
        }
        .emit();
    }
    
    
}

#[near]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            log!("Account @{} burned {}", sender_id, burned_amount);
        }
        used_amount.into()
    }
}

#[near]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            log!("Closed @{} with {}", account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

