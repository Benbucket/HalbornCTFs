use cosmwasm_std::{coins, Addr, Uint128, coin};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use dvamm::asset::AssetInfo;
use dvamm::factory::{ExecuteMsg, InstantiateMsg, PairConfig, PairType};

const DENOM: &str = "uatom";

pub fn mint_tokens(mut app: App, recipient: String, amount: Uint128) -> App {
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: recipient.to_owned(),
            amount: vec![coin(amount.u128(), DENOM)],
        },
    ))
    .unwrap();
    app
}

fn mock_app() -> App {
    App::default()
}

fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = ContractWrapper::new(
        dvamm_factory::contract::execute,
        dvamm_factory::contract::instantiate,
        dvamm_factory::contract::query,
    )
    .with_reply(dvamm_factory::contract::reply);

    app.store_code(Box::new(factory_contract))
}

fn store_pair_code(app: &mut App) -> u64 {
    let pair_contract = ContractWrapper::new(
        dvamm_pair::contract::execute,
        dvamm_pair::contract::instantiate,
        dvamm_pair::contract::query,
    );

    app.store_code(Box::new(pair_contract))
}

fn store_token_code(app: &mut App) -> u64 {
    let token_contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );

    app.store_code(Box::new(token_contract))
}

#[test]
fn test_create_pair_with_same_assets() {
    // Set up the test environment
    let mut app = mock_app();
    
    let owner = Addr::unchecked("owner");
    let user = Addr::unchecked("user");
    
    // Store contract codes
    let factory_code_id = store_factory_code(&mut app);
    let pair_code_id = store_pair_code(&mut app);
    let token_code_id = store_token_code(&mut app);
    
    // Instantiate factory contract
    let factory_instance = app
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &InstantiateMsg {
                pair_xyk_config: Some(PairConfig {
                    code_id: pair_code_id,
                    total_fee_bps: 30,
                    maker_fee_bps: 10,
                }),
                token_code_id,
                owner: owner.to_string(),
                fee_address: None,
            },
            &[],
            "DVAMM Factory",
            None,
        )
        .unwrap();
    
    // Create native token that will be used for the test
    let uatom_asset = AssetInfo::NativeToken { denom: DENOM.to_string() };
    
    // Fund the user with the tokens
    app = mint_tokens(app, user.to_string(), Uint128::from(1000000u128));
    
    // Create a pair with the same asset twice (the vulnerability)
    let create_pair_msg = ExecuteMsg::CreatePair {
        asset_infos: [uatom_asset.clone(), uatom_asset.clone()],
    };
    
    // This should fail with an error in a fixed contract
    // But in a vulnerable contract, this will succeed 
    let result = app.execute_contract(
        user.clone(),
        factory_instance.clone(),
        &create_pair_msg,
        &[],
    );
    
    // In the vulnerable code, this execution will succeed
    assert!(result.is_ok(), "Creating a pair with duplicate assets succeeded, confirming the vulnerability!");
    
    // Additional assertions to verify the vulnerability
    // If the vulnerability exists, we should be able to query the created pair
    // and potentially exploit it further
    
    // You could add additional tests here to demonstrate how this vulnerability
    // can be exploited to withdraw more tokens than fair share
}