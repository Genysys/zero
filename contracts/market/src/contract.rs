#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, wasm_instantiate, Addr, Binary, Deps, DepsMut, Env, Isqrt, MessageInfo, Reply,
    Response, StdError, Storage, SubMsg, Uint128,
};
use cw2::set_contract_version;
use cw_zll_std_liquidity_pool::{ap::Asset, asset::create_coin_asset};
use cw_zll_std_market::{
    borrow::{BorrowingTerms, BorrowingTermsResponse},
    expiry_time::ExpiryTime,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    response::{
        LiquidityPoolResponse, MarketOperatorResponse, MarketPhase, MarketPhaseResponse,
        MarketPhasesInfoResponse,
    },
    state::{
        get_alpha, get_blocks_per_year, get_liquidity_pool, get_market_info, get_market_operator,
        set_config, set_liquidity_pool, Config, MarketPhasesInfo,
    },
};
use cw_zll_std_utils::reply::{parse_reply_instantiate_data, MsgInstantiateContractResponse};

use crate::error::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-zll-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const ADDR_WHILE_INSTANTIATION: &str = "";

/// A `reply` call code ID used for sub-messages.
const INSTANTIATE_LIQUIDITY_POOL_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    set_config(
        deps.storage,
        Config {
            market_operator: msg.market_operator.clone(),
            liquidity_pool: Addr::unchecked(ADDR_WHILE_INSTANTIATION),
            market_phases_info: validate_market_phases_info(msg.market_phases_info)?,
            blocks_per_year: msg.blocks_per_year,
            alpha: msg.alpha,
        },
    )?;

    Ok(Response::new()
        .add_submessage(create_liquidity_pool_contract_instantiate_msg(
            env.contract.address,
            msg.liquidity_pool_code_id,
            msg.liquidity_pool_token_code_id,
            msg.asset_infos,
        )?)
        .add_attributes(vec![
            ("method", "instantiate"),
            ("market_operator", msg.market_operator.to_string().as_ref()),
        ]))
}

fn validate_market_phases_info(
    market_phases_info: MarketPhasesInfo,
) -> Result<MarketPhasesInfo, ContractError> {
    if market_phases_info.market_started_at >= market_phases_info.lp_phase_ends_at {
        return Err(StdError::GenericErr {
            msg: format!(
                "`market_started_at` = {} must occur before `lp_phase_ends_at` = {}",
                &market_phases_info.market_started_at, &market_phases_info.lp_phase_ends_at
            ),
        }
        .into());
    }

    if market_phases_info.lp_phase_ends_at >= market_phases_info.amm_phase_ends_at {
        return Err(StdError::GenericErr {
            msg: format!(
                "`lp_phase_ends_at` = {} must occur before `amm_phase_ends_at` = {}",
                &market_phases_info.lp_phase_ends_at, &market_phases_info.amm_phase_ends_at
            ),
        }
        .into());
    }

    if market_phases_info.amm_phase_ends_at >= market_phases_info.settlement_phase_ends_at {
        return Err(StdError::GenericErr {
            msg: format!(
                "`amm_phase_ends_at` = {} must occur before `settlement_phase_ends_at` = {}",
                &market_phases_info.amm_phase_ends_at, &market_phases_info.settlement_phase_ends_at
            ),
        }
        .into());
    }

    Ok(market_phases_info)
}

fn create_liquidity_pool_contract_instantiate_msg(
    market_contract_addr: Addr,
    liquidity_pool_code_id: u64,
    liquidity_pool_token_code_id: u64,
    asset_infos: [cw_zll_std_liquidity_pool::ap::AssetInfo; 2],
) -> Result<SubMsg, ContractError> {
    Ok(SubMsg::reply_on_success(
        wasm_instantiate(
            liquidity_pool_code_id,
            &cw_zll_std_liquidity_pool::msg::InstantiateMsg {
                asset_infos,
                token_code_id: liquidity_pool_token_code_id,
                factory_addr: market_contract_addr.to_string(),
                init_params: None,
            },
            vec![],
            String::from("ZLL LP"),
        )?,
        INSTANTIATE_LIQUIDITY_POOL_REPLY_ID,
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let market_phase = get_current_market_phase(deps.storage, env.block.height)?;

    match msg {
        ExecuteMsg::Borrow {
            expected_borrow,
            pledged_collateral,
        } => {
            if !market_phase.can_amm_accept_borrowing() {
                return Err(ContractError::Unauthorized {});
            }

            execute_borrow(deps, env, info.sender, expected_borrow, pledged_collateral)
        }
    }
}

fn execute_borrow(
    deps: DepsMut,
    env: Env,
    _borrower: Addr,
    expected_borrow: Asset,
    pledged_collateral: Asset,
) -> Result<Response, ContractError> {
    // TODO: assert if expected_borrow, and pledged_collateral are of the right asset of the LP
    // TODO: validate & accept funds (similar to deposits in the LP flow)
    let BorrowingTerms { borrow, .. } =
        get_borrowing_terms(deps.as_ref(), pledged_collateral, env.block.height)?;

    if expected_borrow.amount > borrow.amount {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Expected amount to borrow ({}) is higher than calculated collateral amount ({})",
            &expected_borrow.amount, &borrow.amount
        ))));
    }

    Ok(Response::default())
}

fn get_borrowing_terms(
    deps: Deps,
    pledged_collateral: Asset,
    current_block_height: u64,
) -> Result<BorrowingTerms, ContractError> {
    let borrowable_amount = get_borrowable_amount(pledged_collateral.clone())?;
    let interest_amount = get_interest_cost(deps, pledged_collateral, current_block_height)?;
    // TODO: return correct asset types (denom/contract address)
    Ok(BorrowingTerms {
        borrow: create_coin_asset(borrowable_amount.u128(), "uluna"),
        interest: create_coin_asset(interest_amount.u128(), "uusd"),
        repayment: create_coin_asset(0, "uusd"),
    })
}

fn get_borrowable_amount(pledged_collateral: Asset) -> Result<Uint128, ContractError> {
    // TODO: query these 3 values below from the LP
    let borrow_ccy_supply = Uint128::new(0);
    let amm_constant = Uint128::new(0);
    let collateral_ccy_supply = Uint128::new(0);

    calculate_borrowable_amount(
        borrow_ccy_supply,
        amm_constant,
        collateral_ccy_supply,
        pledged_collateral.amount,
    )
}

fn calculate_borrowable_amount(
    borrow_ccy_supply: Uint128,
    amm_constant: Uint128,
    collateral_ccy_supply: Uint128,
    collateral_amount: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(borrow_ccy_supply.checked_sub(
        amm_constant.checked_div(collateral_ccy_supply.checked_add(collateral_amount)?)?,
    )?)
}

fn get_interest_cost(
    deps: Deps,
    pledged_collateral: Asset,
    current_block_height: u64,
) -> Result<Uint128, ContractError> {
    let ExpiryTime {
        sqrt_time_to_expiry,
        ..
    } = get_expiry_time(deps.storage, current_block_height)?;

    let oblivious_put_price = get_oblivious_put_price(deps, sqrt_time_to_expiry)?;
    // TODO: read this info form PairInfo via deps query
    let collateral_ccy_decimals = 6;

    calculate_interest_cost(
        oblivious_put_price,
        pledged_collateral.amount,
        collateral_ccy_decimals,
    )
}

fn calculate_interest_cost(
    oblivious_put_price: Uint128,
    collateral_amount: Uint128,
    collateral_ccy_decimals: u8,
) -> Result<Uint128, ContractError> {
    Ok(oblivious_put_price
        .checked_mul(collateral_amount)?
        .checked_div(Uint128::from(10u8).pow(collateral_ccy_decimals.into()))?)
}

fn get_expiry_time(
    storage: &dyn Storage,
    current_block_height: u64,
) -> Result<ExpiryTime, ContractError> {
    let MarketPhasesInfo {
        amm_phase_ends_at, ..
    } = get_market_info(storage)?;
    let blocks_per_year = get_blocks_per_year(storage)?;

    calculate_expiry_time(current_block_height, amm_phase_ends_at, blocks_per_year)
}

fn calculate_expiry_time(
    current_block_height: u64,
    amm_phase_ends_at: u64,
    blocks_per_year: u64,
) -> Result<ExpiryTime, ContractError> {
    let time_to_expiry = Uint128::from(amm_phase_ends_at)
        .checked_sub(current_block_height.into())?
        .checked_div(blocks_per_year.into())?;
    let sqrt_time_to_expiry = time_to_expiry.isqrt();

    Ok(ExpiryTime {
        time_to_expiry,
        sqrt_time_to_expiry,
    })
}

fn get_oblivious_put_price(
    deps: Deps,
    _sqrt_time_to_expiry: Uint128,
) -> Result<Uint128, ContractError> {
    let _alpha = get_alpha(deps.storage)?;
    calculate_oblivious_put_price()
}

fn calculate_oblivious_put_price() -> Result<Uint128, ContractError> {
    // fn calculate_oblivious_put_price(collateral_price: Uint128, collateral_price_annualized_vol: Uint128, sqrt_time_to_expiry: Uint128) -> Result<Uint128, ContractError> {

    // let alpha = get_alpha(deps.storage)?;
    Ok(Uint128::zero())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetMarketOperator {} => query_get_market_operator(deps),
        QueryMsg::GetLiquidityPool {} => query_get_liquidity_pool(deps),
        QueryMsg::GetMarketPhase {} => query_get_market_phase(deps, env),
        QueryMsg::GetMarketPhasesInfo {} => query_get_market_phases_info(deps),
        QueryMsg::GetBorrowingTerms { pledged_collateral } => {
            query_get_borrowing_terms(deps, pledged_collateral, env.block.height)
        }
    }
}

fn query_get_market_phase(deps: Deps, env: Env) -> Result<Binary, ContractError> {
    let response = MarketPhaseResponse {
        phase: get_current_market_phase(deps.storage, env.block.height)?,
    };

    Ok(to_binary(&response)?)
}

fn query_get_market_phases_info(deps: Deps) -> Result<Binary, ContractError> {
    let response: MarketPhasesInfoResponse = get_market_info(deps.storage)?.into();

    Ok(to_binary(&response)?)
}

fn query_get_market_operator(deps: Deps) -> Result<Binary, ContractError> {
    let response = MarketOperatorResponse {
        market_operator: get_market_operator(deps.storage)?,
    };

    Ok(to_binary(&response)?)
}

fn query_get_liquidity_pool(deps: Deps) -> Result<Binary, ContractError> {
    let response = LiquidityPoolResponse {
        liquidity_pool: get_liquidity_pool(deps.storage)?,
    };

    Ok(to_binary(&response)?)
}

fn get_current_market_phase(
    storage: &dyn Storage,
    current_block_height: u64,
) -> Result<MarketPhase, ContractError> {
    let market_phases_info = get_market_info(storage)?;

    if current_block_height <= market_phases_info.lp_phase_ends_at {
        return Ok(MarketPhase::ProvidingLiquidity);
    }

    if current_block_height <= market_phases_info.amm_phase_ends_at {
        return Ok(MarketPhase::AutomatedMarketMaker);
    }

    if current_block_height <= market_phases_info.settlement_phase_ends_at {
        return Ok(MarketPhase::Settlement);
    }

    Ok(MarketPhase::PostSettlement)
}

fn query_get_borrowing_terms(
    deps: Deps,
    pledged_collateral: Asset,
    current_block_height: u64,
) -> Result<Binary, ContractError> {
    let response: BorrowingTermsResponse =
        get_borrowing_terms(deps, pledged_collateral, current_block_height)?.into();

    Ok(to_binary(&response)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.result.is_err() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: msg.result.unwrap_err(),
        }));
    }

    match msg.id {
        INSTANTIATE_LIQUIDITY_POOL_REPLY_ID => reply_on_instantiate_liquidity_pool(deps, env, msg),
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("reply id `{:?}` is invalid", msg.id),
        })),
    }
}

fn reply_on_instantiate_liquidity_pool(
    deps: DepsMut,
    _env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    if get_liquidity_pool(deps.storage)? != Addr::unchecked(ADDR_WHILE_INSTANTIATION) {
        return Err(ContractError::Unauthorized {});
    }

    let response: MsgInstantiateContractResponse = parse_reply_instantiate_data(msg)?;

    let Config { liquidity_pool, .. } = set_liquidity_pool(
        deps.storage,
        deps.api.addr_validate(&response.contract_address)?,
    )?;

    Ok(Response::new().add_attribute("liquidity_pool_addr", liquidity_pool))
}
