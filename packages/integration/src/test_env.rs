use crate::{
    consts::{
        ADDR_ADMIN, ADDR_LIQUIDITY_PROVIDER, ADDR_LIQUIDITY_PROVIDER_2, ADDR_MARKET_OPERATOR,
        BLOCK_HEIGHT_AMM_ENDS_AT, BLOCK_HEIGHT_LP_ENDS_AT, BLOCK_HEIGHT_MARKET_STARTS_AT,
        BLOCK_HEIGHT_SETTLEMENT_ENDS_AT,
    },
    contract_helpers::ContractBase,
    contract_mocks::{
        Cw20TokenContract, LiquidityPoolContract, LiquidityPoolTokenContract, MarketContract,
    },
    terra_querier_mock::TerraCustomQueryHandler,
};
use anyhow::Result;
use cosmwasm_std::{coin, to_binary, Addr, Empty, Uint128};
use cw20::{Cw20Coin, Cw20ExecuteMsg};
use cw_multi_test::{App, AppBuilder, AppResponse, Executor};
use cw_zll_std_liquidity_pool::ap::{Asset, AssetInfo, PairInfo};
use cw_zll_std_market::{response::LiquidityPoolResponse, state::MarketPhasesInfo};
use terra_cosmwasm::TerraQueryWrapper;

pub fn mock_app() -> App<Empty, TerraQueryWrapper> {
    let custom_handler = TerraCustomQueryHandler;

    AppBuilder::new().with_custom(custom_handler).build()
}

pub struct MarketSetup {
    pub market_contract: MarketContract,
    pub liquidity_pool_contract: LiquidityPoolContract,
}

pub fn create_martket_setup(
    app: &mut App<Empty, TerraQueryWrapper>,
    asset_infos: [AssetInfo; 2],
) -> MarketSetup {
    let liquidity_pool_code_id = app.store_code(LiquidityPoolContract::contract_code());
    let liquidity_pool_token_code_id = app.store_code(LiquidityPoolTokenContract::contract_code());
    let market_contract_code_id = app.store_code(MarketContract::contract_code());

    let market_start_at = BLOCK_HEIGHT_MARKET_STARTS_AT;

    app.update_block(|block| {
        block.height = market_start_at;
    });

    let market_contract_addr = app
        .instantiate_contract(
            market_contract_code_id,
            Addr::unchecked(ADDR_ADMIN),
            &cw_zll_std_market::msg::InstantiateMsg {
                market_operator: Addr::unchecked(ADDR_MARKET_OPERATOR),
                asset_infos,
                liquidity_pool_code_id,
                liquidity_pool_token_code_id,
                market_phases_info: MarketPhasesInfo {
                    market_started_at: market_start_at,
                    lp_phase_ends_at: BLOCK_HEIGHT_LP_ENDS_AT,
                    amm_phase_ends_at: BLOCK_HEIGHT_AMM_ENDS_AT,
                    settlement_phase_ends_at: BLOCK_HEIGHT_SETTLEMENT_ENDS_AT,
                },
                blocks_per_year: 4_204_800, // assuming one block per 7.5 seconds
                alpha: 200_000_000_000,
            },
            &[],
            "ZLL Market",
            Some(ADDR_ADMIN.into()),
        )
        .unwrap();

    let market_contract = MarketContract(market_contract_addr);

    let response: LiquidityPoolResponse = app
        .wrap()
        .query_wasm_smart(
            market_contract.addr(),
            &cw_zll_std_market::msg::QueryMsg::GetLiquidityPool {},
        )
        .unwrap();

    let liquidity_pool_addr = response.liquidity_pool;

    let response: PairInfo = app
        .wrap()
        .query_wasm_smart(
            liquidity_pool_addr.clone(),
            &cw_zll_std_liquidity_pool::msg::QueryMsg::Pair {},
        )
        .unwrap();

    let liquidity_pool_token_addr = response.liquidity_token;

    let liquidity_pool_contract =
        LiquidityPoolContract(liquidity_pool_addr, liquidity_pool_token_addr);

    MarketSetup {
        market_contract,
        liquidity_pool_contract,
    }
}

pub fn create_cw20_token(app: &mut App<Empty, TerraQueryWrapper>) -> Cw20TokenContract {
    // deploy custom CW20 token that will serve as one of the pool's assets
    let cw20_token_code_id = app.store_code(Cw20TokenContract::contract_code());
    let cw20_token_addr = app
        .instantiate_contract(
            cw20_token_code_id,
            Addr::unchecked(ADDR_ADMIN),
            &cw20_base::msg::InstantiateMsg {
                name: "Custom Pool Asset".into(),
                symbol: "CPA".into(),
                decimals: 9,
                initial_balances: vec![
                    Cw20Coin {
                        address: ADDR_LIQUIDITY_PROVIDER.into(),
                        amount: Uint128::new(100_000_000_000),
                    },
                    Cw20Coin {
                        address: ADDR_LIQUIDITY_PROVIDER_2.into(),
                        amount: Uint128::new(250_000_000_000),
                    },
                ],
                mint: None,
                marketing: None,
            },
            &[],
            "ZLL Pool Asset Token",
            None,
        )
        .unwrap();

    Cw20TokenContract(cw20_token_addr)
}

pub fn try_to_withdraw_liquidity(
    app: &mut App<Empty, TerraQueryWrapper>,
    _market_contract: &MarketContract,
    liquidity_pool_contract: &LiquidityPoolContract,
) -> Result<AppResponse, anyhow::Error> {
    let PairInfo {
        liquidity_token, ..
    } = app
        .wrap()
        .query_wasm_smart(
            liquidity_pool_contract.addr(),
            &cw_zll_std_liquidity_pool::msg::QueryMsg::Pair {},
        )
        .unwrap();

    let provider_lp_token_balance = cw20::Cw20Contract(liquidity_token.clone())
        .balance(app, ADDR_LIQUIDITY_PROVIDER)
        .unwrap();

    let send_lp_tokens_to_for_burning_msg = cw20::Cw20Contract(liquidity_token)
        .call(Cw20ExecuteMsg::Send {
            contract: liquidity_pool_contract.addr().to_string(),
            amount: provider_lp_token_balance,
            msg: to_binary(&cw_zll_std_liquidity_pool::msg::Cw20HookMsg::WithdrawLiquidity {})
                .unwrap(),
        })
        .unwrap();

    app.execute(
        Addr::unchecked(ADDR_LIQUIDITY_PROVIDER),
        send_lp_tokens_to_for_burning_msg,
    )
}

pub fn try_to_deposit_liquidity(
    app: &mut App<Empty, TerraQueryWrapper>,
    liquidity_pool_contract: &LiquidityPoolContract,
    addr_liquidity_provider: &str,
    assets_to_provide_as_liquidity: [Asset; 2],
) -> Result<Vec<AppResponse>> {
    let coins = assets_to_provide_as_liquidity
        .iter()
        .filter_map(|asset| match &asset.info {
            AssetInfo::NativeToken { denom } => Some(coin(asset.amount.u128(), denom)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let allowance_msgs = assets_to_provide_as_liquidity
        .iter()
        .filter_map(|asset| match &asset.info {
            AssetInfo::Token { contract_addr } => Some(
                cw20::Cw20Contract(contract_addr.to_owned())
                    .call(Cw20ExecuteMsg::IncreaseAllowance {
                        spender: liquidity_pool_contract.addr().to_string(),
                        amount: asset.amount,
                        expires: None,
                    })
                    .unwrap(),
            ),
            _ => None,
        })
        .collect::<Vec<_>>();

    let _ = app.init_bank_balance(&Addr::unchecked(addr_liquidity_provider), coins.clone());

    let deposit_msg = liquidity_pool_contract
        .call(
            &cw_zll_std_liquidity_pool::msg::ExecuteMsg::ProvideLiquidity {
                assets: assets_to_provide_as_liquidity,
                slippage_tolerance: None,
                auto_stake: None,
                receiver: None,
            },
            Some(coins),
        )
        .unwrap();

    let mut messages = allowance_msgs;
    messages.push(deposit_msg);

    app.execute_multi(Addr::unchecked(addr_liquidity_provider), messages)
}
