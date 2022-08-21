use cw_zll_std_integration::{
    consts::{ADDR_LIQUIDITY_PROVIDER, BLOCK_HEIGHT_LP_ENDS_AT},
    test_env::{self, MarketSetup},
};
use cw_zll_std_liquidity_pool::asset::create_coin_asset;
use cw_zll_std_market::response::MarketPhase;

#[test]
fn all_handlers_from_other_phases_are_not_avaialable() {
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
        market_contract,
        liquidity_pool_contract,
    } = test_env::create_martket_setup(&mut app, asset_infos);

    // Last block of the LP phase
    app.update_block(|block| {
        block.height = BLOCK_HEIGHT_LP_ENDS_AT;
    });

    assert_eq!(
        market_contract.get_market_phase(&app).unwrap(),
        MarketPhase::ProvidingLiquidity
    );

    let response = test_env::try_to_deposit_liquidity(
        &mut app,
        &liquidity_pool_contract,
        ADDR_LIQUIDITY_PROVIDER,
        assets_to_provide_as_liquidity.clone(),
    );

    if response.is_err() {
        println!("{:?}", &response);
    }

    assert_eq!(
        response.is_ok(),
        true,
        "depositing liquidity must be possible in the providing liquidity phase of the market"
    );

    // First block of the AMM phase
    app.update_block(|block| {
        block.height = BLOCK_HEIGHT_LP_ENDS_AT + 1;
    });

    let market_phase = market_contract.get_market_phase(&app).unwrap();

    assert_eq!(market_phase, MarketPhase::AutomatedMarketMaker);

    let response = test_env::try_to_deposit_liquidity(
        &mut app,
        &liquidity_pool_contract,
        ADDR_LIQUIDITY_PROVIDER,
        assets_to_provide_as_liquidity,
    );

    assert_eq!(
        response.is_err(),
        true,
        "depositing liquidity is only possible in the providing liquidity phase of the market"
    );

    let response =
        test_env::try_to_withdraw_liquidity(&mut app, &market_contract, &liquidity_pool_contract);

    assert_eq!(
        response.is_err(),
        true,
        "withdrawing liquidity is only possible in the post-settlement phase of the market"
    );
}
