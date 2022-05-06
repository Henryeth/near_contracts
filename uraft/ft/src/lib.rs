/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg width='373' height='373' viewBox='0 0 373 373' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cg filter='url(%23filter0_ii_2016_186744)'%3E%3Ccircle cx='186.432' cy='186.432' r='173.432' fill='url(%23paint0_linear_2016_186744)'/%3E%3C/g%3E%3Ccircle cx='186.432' cy='186.432' r='179.856' stroke='url(%23paint1_linear_2016_186744)' stroke-width='12.8468'/%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M218.981 204.243H231.074L272.216 233.785H292.045L251.038 204.243H259.536C264.033 204.243 268.259 203.389 272.216 201.68C276.173 199.972 279.59 197.679 282.468 194.801C285.435 191.833 287.774 188.371 289.482 184.414C291.191 180.457 292.045 176.231 292.045 171.734C292.045 167.238 291.191 163.056 289.482 159.189C287.774 155.233 285.435 151.815 282.468 148.938C279.59 145.97 276.173 143.632 272.216 141.923C268.259 140.215 264.033 139.36 259.536 139.36H174.015V151.096H259.536C262.414 151.096 265.112 151.636 267.63 152.715C270.148 153.794 272.306 155.278 274.105 157.166C275.993 159.055 277.477 161.258 278.556 163.776C279.635 166.204 280.175 168.857 280.175 171.734C280.175 174.612 279.635 177.31 278.556 179.828C277.477 182.346 275.993 184.549 274.105 186.438C272.306 188.326 270.148 189.81 267.63 190.889C265.112 191.968 262.414 192.508 259.536 192.508H203.439L218.981 204.243Z' fill='white'/%3E%3Cpath d='M231.074 204.243L234.821 199.026L233.141 197.82H231.074V204.243ZM218.981 204.243L215.11 209.37L216.828 210.667H218.981V204.243ZM272.216 233.785L268.47 239.002L270.149 240.208H272.216V233.785ZM292.045 233.785V240.208H311.951L295.8 228.573L292.045 233.785ZM251.038 204.243V197.82H231.132L247.283 209.455L251.038 204.243ZM272.216 201.68L274.763 207.578L274.763 207.578L272.216 201.68ZM282.468 194.801L287.01 199.343V199.343L282.468 194.801ZM289.482 184.414L283.585 181.868V181.868L289.482 184.414ZM289.482 159.189L283.585 161.736L283.596 161.761L283.607 161.786L289.482 159.189ZM282.468 148.938L277.856 153.409L277.925 153.48L277.996 153.549L282.468 148.938ZM272.216 141.923L274.763 136.026L274.763 136.026L272.216 141.923ZM174.015 139.36V132.937H167.591V139.36H174.015ZM174.015 151.096H167.591V157.519H174.015V151.096ZM267.63 152.715L265.099 158.619L265.099 158.619L267.63 152.715ZM274.105 157.166L269.453 161.596L269.507 161.653L269.563 161.708L274.105 157.166ZM278.556 163.776L272.652 166.306L272.669 166.345L272.686 166.385L278.556 163.776ZM274.105 186.438L269.563 181.896L269.507 181.951L269.453 182.008L274.105 186.438ZM267.63 190.889L265.099 184.985L265.099 184.985L267.63 190.889ZM203.439 192.508V186.084H184.273L199.568 197.634L203.439 192.508ZM231.074 197.82H218.981V210.667H231.074V197.82ZM275.963 228.567L234.821 199.026L227.328 209.461L268.47 239.002L275.963 228.567ZM292.045 227.361H272.216V240.208H292.045V227.361ZM247.283 209.455L288.291 238.997L295.8 228.573L254.793 199.032L247.283 209.455ZM259.536 197.82H251.038V210.667H259.536V197.82ZM269.67 195.783C266.552 197.13 263.195 197.82 259.536 197.82V210.667C264.87 210.667 269.967 209.648 274.763 207.578L269.67 195.783ZM277.926 190.259C275.655 192.53 272.926 194.377 269.67 195.783L274.763 207.578C279.42 205.566 283.525 202.828 287.01 199.343L277.926 190.259ZM283.585 181.868C282.189 185.1 280.304 187.88 277.926 190.259L287.01 199.343C290.567 195.786 293.358 191.642 295.379 186.961L283.585 181.868ZM285.622 171.734C285.622 175.394 284.932 178.75 283.585 181.868L295.379 186.961C297.45 182.165 298.469 177.068 298.469 171.734H285.622ZM283.607 161.786C284.931 164.783 285.622 168.074 285.622 171.734H298.469C298.469 166.402 297.45 161.329 295.358 156.593L283.607 161.786ZM277.996 153.549C280.332 155.814 282.197 158.521 283.585 161.736L295.379 156.643C293.35 151.945 290.539 147.817 286.939 144.326L277.996 153.549ZM269.67 147.82C272.885 149.209 275.592 151.074 277.856 153.409L287.079 144.466C283.589 140.866 279.461 138.055 274.763 136.026L269.67 147.82ZM259.536 145.784C263.195 145.784 266.552 146.474 269.67 147.82L274.763 136.026C269.967 133.955 264.87 132.937 259.536 132.937V145.784ZM174.015 145.784H259.536V132.937H174.015V145.784ZM180.438 151.096V139.36H167.591V151.096H180.438ZM259.536 144.673H174.015V157.519H259.536V144.673ZM270.16 146.811C266.802 145.372 263.239 144.673 259.536 144.673V157.519C261.589 157.519 263.421 157.899 265.099 158.619L270.16 146.811ZM278.756 152.736C276.327 150.186 273.437 148.215 270.16 146.811L265.099 158.619C266.859 159.373 268.285 160.369 269.453 161.596L278.756 152.736ZM284.46 161.245C283.067 157.996 281.128 155.105 278.647 152.624L269.563 161.708C270.858 163.004 271.886 164.519 272.652 166.306L284.46 161.245ZM286.598 171.734C286.598 168.034 285.899 164.483 284.426 161.167L272.686 166.385C273.371 167.925 273.751 169.68 273.751 171.734H286.598ZM284.46 182.358C285.899 179.001 286.598 175.437 286.598 171.734H273.751C273.751 173.787 273.371 175.619 272.652 177.298L284.46 182.358ZM278.647 190.98C281.128 188.498 283.067 185.608 284.46 182.358L272.652 177.298C271.886 179.084 270.858 180.6 269.563 181.896L278.647 190.98ZM270.16 196.793C273.437 195.389 276.327 193.418 278.756 190.868L269.453 182.008C268.285 183.235 266.859 184.231 265.099 184.985L270.16 196.793ZM259.536 198.931C263.239 198.931 266.802 198.232 270.16 196.793L265.099 184.985C263.421 185.704 261.589 186.084 259.536 186.084V198.931ZM203.439 198.931H259.536V186.084H203.439V198.931ZM199.568 197.634L215.11 209.37L222.852 199.117L207.31 187.382L199.568 197.634Z' fill='white'/%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M183.412 209.779C183.239 211.107 182.873 212.393 182.314 213.635C181.595 215.434 180.561 217.007 179.212 218.356C177.863 219.705 176.289 220.784 174.49 221.594C172.692 222.313 170.758 222.673 168.69 222.673H103.807C101.829 222.673 99.9402 222.313 98.1416 221.594C96.3431 220.784 94.7693 219.705 93.4204 218.356C92.0715 217.007 90.9924 215.434 90.183 213.635C89.4636 211.837 89.1039 209.948 89.1039 207.97V140.119H77.2334V207.97C77.2334 211.657 77.9079 215.119 79.2568 218.356C80.6956 221.594 82.5841 224.426 84.9222 226.855C87.3503 229.193 90.183 231.081 93.4204 232.52C96.6578 233.869 100.12 234.543 103.807 234.543H168.69C172.377 234.543 175.839 233.869 179.077 232.52C182.314 231.081 185.102 229.193 187.44 226.855C189.868 224.426 191.757 221.594 193.106 218.356C193.221 218.098 193.331 217.837 193.437 217.576L183.412 209.779Z' fill='white'/%3E%3Cpath d='M182.314 213.635L176.457 210.999L176.401 211.123L176.35 211.25L182.314 213.635ZM183.412 209.779L187.356 204.708L178.491 197.814L177.042 208.95L183.412 209.779ZM179.212 218.356L183.754 222.898L183.754 222.898L179.212 218.356ZM174.49 221.594L176.876 227.558L177.002 227.507L177.126 227.451L174.49 221.594ZM98.1416 221.594L95.5057 227.451L95.6298 227.507L95.7561 227.558L98.1416 221.594ZM93.4204 218.356L88.8784 222.898L88.8784 222.898L93.4204 218.356ZM90.183 213.635L84.219 216.021L84.2695 216.147L84.3254 216.271L90.183 213.635ZM89.1039 140.119H95.5273V133.696H89.1039V140.119ZM77.2334 140.119V133.696H70.81V140.119H77.2334ZM79.2568 218.356L73.3275 220.827L73.3564 220.896L73.387 220.965L79.2568 218.356ZM84.9222 226.855L80.2953 231.31L80.3794 231.397L80.4667 231.481L84.9222 226.855ZM93.4204 232.52L90.8116 238.39L90.8804 238.42L90.9499 238.449L93.4204 232.52ZM179.077 232.52L181.547 238.449L181.617 238.42L181.686 238.39L179.077 232.52ZM193.106 218.356L187.236 215.748L187.205 215.816L187.176 215.886L193.106 218.356ZM193.437 217.576L199.392 219.984L201.211 215.485L197.38 212.505L193.437 217.576ZM188.172 216.271C188.985 214.464 189.526 212.571 189.782 210.607L177.042 208.95C176.952 209.643 176.762 210.321 176.457 210.999L188.172 216.271ZM183.754 222.898C185.727 220.925 187.241 218.614 188.278 216.021L176.35 211.25C175.949 212.253 175.394 213.09 174.67 213.814L183.754 222.898ZM177.126 227.451C179.61 226.334 181.836 224.816 183.754 222.898L174.67 213.814C173.89 214.594 172.968 215.235 171.855 215.736L177.126 227.451ZM168.69 229.096C171.52 229.096 174.269 228.601 176.876 227.558L172.105 215.63C171.115 216.026 169.997 216.249 168.69 216.249V229.096ZM103.807 229.096H168.69V216.249H103.807V229.096ZM95.7561 227.558C98.3297 228.587 101.029 229.096 103.807 229.096V216.249C102.628 216.249 101.551 216.039 100.527 215.63L95.7561 227.558ZM88.8784 222.898C90.7963 224.816 93.0217 226.334 95.5057 227.451L100.778 215.736C99.6644 215.235 98.7424 214.594 97.9625 213.814L88.8784 222.898ZM84.3254 216.271C85.4431 218.755 86.9604 220.98 88.8784 222.898L97.9625 213.814C97.1826 213.034 96.5416 212.112 96.0407 210.999L84.3254 216.271ZM82.6805 207.97C82.6805 210.747 83.1896 213.447 84.219 216.021L96.147 211.25C95.7376 210.226 95.5273 209.149 95.5273 207.97H82.6805ZM82.6805 140.119V207.97H95.5273V140.119H82.6805ZM77.2334 146.542H89.1039V133.696H77.2334V146.542ZM83.6568 207.97V140.119H70.81V207.97H83.6568ZM85.1861 215.886C84.1799 213.471 83.6568 210.853 83.6568 207.97H70.81C70.81 212.461 71.6358 216.767 73.3275 220.827L85.1861 215.886ZM89.5492 222.399C87.7557 220.537 86.2774 218.337 85.1266 215.748L73.387 220.965C75.1138 224.851 77.4125 228.316 80.2953 231.31L89.5492 222.399ZM96.0292 226.65C93.4398 225.499 91.2402 224.021 89.3778 222.228L80.4667 231.481C83.4604 234.364 86.9262 236.663 90.8116 238.39L96.0292 226.65ZM103.807 228.12C100.924 228.12 98.3057 227.597 95.891 226.591L90.9499 238.449C95.0099 240.141 99.3162 240.967 103.807 240.967V228.12ZM168.69 228.12H103.807V240.967H168.69V228.12ZM176.606 226.591C174.191 227.597 171.573 228.12 168.69 228.12V240.967C173.181 240.967 177.487 240.141 181.547 238.449L176.606 226.591ZM182.898 222.312C181.156 224.054 179.037 225.509 176.468 226.65L181.686 238.39C185.592 236.654 189.048 234.331 191.982 231.397L182.898 222.312ZM187.176 215.886C186.143 218.366 184.72 220.491 182.898 222.312L191.982 231.397C195.016 228.362 197.37 224.822 199.035 220.827L187.176 215.886ZM187.482 215.167C187.403 215.361 187.321 215.555 187.236 215.748L198.975 220.965C199.12 220.64 199.259 220.313 199.392 219.984L187.482 215.167ZM197.38 212.505L187.356 204.708L179.469 214.849L189.493 222.646L197.38 212.505Z' fill='white'/%3E%3Cdefs%3E%3Cfilter id='filter0_ii_2016_186744' x='0.153198' y='-24.8468' width='372.558' height='423.252' filterUnits='userSpaceOnUse' color-interpolation-filters='sRGB'%3E%3CfeFlood flood-opacity='0' result='BackgroundImageFix'/%3E%3CfeBlend mode='normal' in='SourceGraphic' in2='BackgroundImageFix' result='shape'/%3E%3CfeColorMatrix in='SourceAlpha' type='matrix' values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0' result='hardAlpha'/%3E%3CfeOffset dy='25.6937'/%3E%3CfeGaussianBlur stdDeviation='12.8468'/%3E%3CfeComposite in2='hardAlpha' operator='arithmetic' k2='-1' k3='1'/%3E%3CfeColorMatrix type='matrix' values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.25 0'/%3E%3CfeBlend mode='normal' in2='shape' result='effect1_innerShadow_2016_186744'/%3E%3CfeColorMatrix in='SourceAlpha' type='matrix' values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0' result='hardAlpha'/%3E%3CfeOffset dy='-25'/%3E%3CfeGaussianBlur stdDeviation='12.5'/%3E%3CfeComposite in2='hardAlpha' operator='arithmetic' k2='-1' k3='1'/%3E%3CfeColorMatrix type='matrix' values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.25 0'/%3E%3CfeBlend mode='normal' in2='effect1_innerShadow_2016_186744' result='effect2_innerShadow_2016_186744'/%3E%3C/filter%3E%3ClinearGradient id='paint0_linear_2016_186744' x1='186.432' y1='13' x2='186.432' y2='359.865' gradientUnits='userSpaceOnUse'%3E%3Cstop stop-color='%234388DD'/%3E%3Cstop offset='1' stop-color='%23C60286'/%3E%3C/linearGradient%3E%3ClinearGradient id='paint1_linear_2016_186744' x1='186.432' y1='13' x2='186.432' y2='359.865' gradientUnits='userSpaceOnUse'%3E%3Cstop stop-color='white'/%3E%3Cstop offset='0.494792' stop-color='%23808080'/%3E%3Cstop offset='1' stop-color='white'/%3E%3C/linearGradient%3E%3C/defs%3E%3C/svg%3E%0A";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "URA fungible token".to_string(),
                symbol: "URA".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 6,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
