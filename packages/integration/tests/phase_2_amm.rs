use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Executor};
use cw_zll_std_integration::{
    consts::{ADDR_LIQUIDITY_PROVIDER, ADDR_REGULAR_USER, BLOCK_HEIGHT_LP_ENDS_AT},
    contract_helpers::ContractBase,
    test_env::{self, MarketSetup},
};
use cw_zll_std_liquidity_pool::asset::create_coin_asset;
use cw_zll_std_market::borrow::BorrowingTermsResponse;
use terra_cosmwasm::TerraQueryWrapper;

#[test]
fn borrower_can_borrow() {
    let mut app = test_env::mock_app();

    let MarketSetup {
        market_contract, ..
    } = setup_market_past_providing_liquidity_phase(&mut app);

    let pledged_collateral = create_coin_asset(111_000_000, "uluna");

    let BorrowingTermsResponse { borrow, .. } = app
        .wrap()
        .query_wasm_smart(
            market_contract.addr(),
            &cw_zll_std_market::msg::QueryMsg::GetBorrowingTerms {
                pledged_collateral: pledged_collateral.clone(),
            },
        )
        .unwrap();

    let response = app.execute(
        Addr::unchecked(ADDR_REGULAR_USER),
        market_contract
            .call(
                &cw_zll_std_market::msg::ExecuteMsg::Borrow {
                    expected_borrow: borrow,
                    pledged_collateral,
                },
                None,
            )
            .unwrap(),
    );

    println!("{:?}", &response);

    assert_eq!(
        response.is_ok(),
        true,
        "Borrower is able to borrow during the AMM phase"
    )
}

#[test]
#[ignore]
fn borrower_can_check_borrowing_terms() {
    todo!()
}

#[test]
#[ignore]
fn lender_can_lend() {
    todo!()
}

#[test]
#[ignore]
fn lender_can_check_lending_terms() {
    todo!()
}

#[test]
#[ignore]
fn anyone_can_check_put_option_pricing_params() {
    todo!()
}

#[test]
#[ignore]
fn anyone_can_check_time_to_expiry() {
    todo!()
}

#[test]
#[ignore]
fn only_market_operator_can_update_put_option_pricing_params() {
    todo!()
}

fn setup_market_past_providing_liquidity_phase(
    mut app: &mut App<Empty, TerraQueryWrapper>,
) -> MarketSetup {
    let list_of_assets_to_provide_as_liquidity = vec![
        [
            create_coin_asset(2_000_000, "uluna"),  // 2 LUNA
            create_coin_asset(500_000_000, "uusd"), // 500 UST
        ],
        [
            create_coin_asset(5_000_000, "uluna"),    // 5 LUNA
            create_coin_asset(5_000_000_000, "uusd"), // 5000 UST
        ],
        [
            create_coin_asset(20_000_000, "uluna"),    // 20 LUNA
            create_coin_asset(30_000_000_000, "uusd"), // 30_000 UST
        ],
        [
            create_coin_asset(200_000_000, "uluna"),    // 200 LUNA
            create_coin_asset(500_000_000_000, "uusd"), // 500_000 UST
        ],
    ];

    let asset_infos = list_of_assets_to_provide_as_liquidity
        .first()
        .unwrap()
        .clone()
        .map(|asset| asset.info);

    let market_setup = test_env::create_martket_setup(&mut app, asset_infos);

    for assets_to_provide_as_liquidity in list_of_assets_to_provide_as_liquidity {
        test_env::try_to_deposit_liquidity(
            &mut app,
            &market_setup.liquidity_pool_contract,
            ADDR_LIQUIDITY_PROVIDER,
            assets_to_provide_as_liquidity.clone(),
        )
        .unwrap();
    }

    app.update_block(|block| {
        block.height = BLOCK_HEIGHT_LP_ENDS_AT + 1;
    });

    market_setup
}
