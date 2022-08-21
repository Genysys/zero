use cw_zll_std_integration::{
    consts::{ADDR_LIQUIDITY_PROVIDER, ADDR_MARKET_OPERATOR},
    contract_helpers::ContractBase,
    test_env::{self, MarketSetup},
};
use cw_zll_std_liquidity_pool::{
    ap::AssetInfo,
    asset::{create_coin_asset, create_token_asset},
};
use cw_zll_std_market::response::{LiquidityPoolResponse, MarketOperatorResponse};

#[test]
fn admin_can_create_a_new_market_with_designated_market_operator() {
    let mut app = test_env::mock_app();

    let MarketSetup {
        market_contract, ..
    } = test_env::create_martket_setup(
        &mut app,
        [
            AssetInfo::NativeToken {
                denom: "uluna".into(),
            },
            AssetInfo::NativeToken {
                denom: "uusd".into(),
            },
        ],
    );

    let response: MarketOperatorResponse = app
        .wrap()
        .query_wasm_smart(
            market_contract.addr(),
            &cw_zll_std_market::msg::QueryMsg::GetMarketOperator {},
        )
        .unwrap();

    assert_eq!(response.market_operator.to_string(), ADDR_MARKET_OPERATOR);
}

#[test]
fn a_liquidity_pool_contract_is_created_automatically_for_a_new_market_contract() {
    let mut app = test_env::mock_app();

    let MarketSetup {
        market_contract, ..
    } = test_env::create_martket_setup(
        &mut app,
        [
            AssetInfo::NativeToken {
                denom: "uluna".into(),
            },
            AssetInfo::NativeToken {
                denom: "uusd".into(),
            },
        ],
    );

    let response: LiquidityPoolResponse = app
        .wrap()
        .query_wasm_smart(
            market_contract.addr(),
            &cw_zll_std_market::msg::QueryMsg::GetLiquidityPool {},
        )
        .unwrap();

    assert_eq!(response.liquidity_pool.to_string().is_empty(), false);
}

#[test]
fn a_new_liquidity_pool_contract_is_able_to_take_deposits_in_native_coins() {
    let mut app = test_env::mock_app();

    //provide 500 UST & 2 LUNA
    let assets_to_provide_as_liquidity = [
        create_coin_asset(2_000_000, "uluna"),  // 2 LUNA
        create_coin_asset(500_000_000, "uusd"), // 500 UST
    ];

    let asset_infos = assets_to_provide_as_liquidity
        .clone()
        .map(|asset| asset.info);

    let MarketSetup {
        liquidity_pool_contract,
        ..
    } = test_env::create_martket_setup(&mut app, asset_infos);

    let response = test_env::try_to_deposit_liquidity(
        &mut app,
        &liquidity_pool_contract,
        ADDR_LIQUIDITY_PROVIDER,
        assets_to_provide_as_liquidity,
    );

    assert_eq!(response.is_ok(), true);
}

#[test]
fn a_new_liquidity_pool_contract_is_able_to_take_deposits_in_cw20_tokens() {
    let mut app = test_env::mock_app();

    // deploy custom CW20 token that will serve as one of the pool's assets
    let cw20_token_contract = test_env::create_cw20_token(&mut app);

    // prepare LP asset information (a pair of two tokens)
    let assets_to_provide_as_liquidity = [
        create_token_asset(2_000_000_000, cw20_token_contract.addr()), // 2 CPT
        create_coin_asset(450_000_000, "uusd"),                        // 450 UST
    ];

    let asset_infos = assets_to_provide_as_liquidity
        .clone()
        .map(|asset| asset.info);

    let MarketSetup {
        liquidity_pool_contract,
        ..
    } = test_env::create_martket_setup(&mut app, asset_infos);

    let response = test_env::try_to_deposit_liquidity(
        &mut app,
        &liquidity_pool_contract,
        ADDR_LIQUIDITY_PROVIDER,
        assets_to_provide_as_liquidity,
    );

    assert_eq!(response.is_ok(), true);
}

#[test]
#[ignore]
fn liquidity_pool_provider_can_provide_assets_to_the_pool_only_with_correct_ratio() {
    let mut app = test_env::mock_app();

    // deploy custom CW20 token that will serve as one of the pool's assets
    let cw20_token_contract = test_env::create_cw20_token(&mut app);

    // prepare LP asset information (a pair of two tokens)
    let assets_to_provide_as_liquidity = [
        create_token_asset(2_000_000_000, cw20_token_contract.addr()), // 2 CPT
        create_coin_asset(450_000_000, "uusd"),                        // 450 UST
    ];

    let asset_infos = assets_to_provide_as_liquidity
        .clone()
        .map(|asset| asset.info);

    let MarketSetup {
        liquidity_pool_contract,
        ..
    } = test_env::create_martket_setup(&mut app, asset_infos);

    let response = test_env::try_to_deposit_liquidity(
        &mut app,
        &liquidity_pool_contract,
        ADDR_LIQUIDITY_PROVIDER,
        assets_to_provide_as_liquidity,
    );

    assert_eq!(response.is_ok(), true);
}
