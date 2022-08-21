#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, wasm_execute, wasm_instantiate, Addr, Binary, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw_zll_std_market::response::MarketPhaseResponse;
use cw_zll_std_utils::reply::{parse_reply_instantiate_data, MsgInstantiateContractResponse};

use crate::{
    error::ContractError,
    state::{Config, CONFIG},
};

use cw_zll_std_liquidity_pool::{
    ap::{
        asset::{format_lp_token_name, Asset, AssetInfo, PairInfo},
        factory::PairType,
        pair::{Cw20HookMsg, PoolResponse},
        querier::query_supply,
        U256,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInstantiateMsg},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-zll-liquidity-pool";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A `reply` call code ID used for sub-messages.
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

const ADDR_WHILE_INSTANTIATION: &str = "";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.asset_infos[0].check(deps.api)?;
    msg.asset_infos[1].check(deps.api)?;

    if msg.asset_infos[0] == msg.asset_infos[1] {
        return Err(ContractError::DoublingAssets {});
    }

    let config = Config {
        pair_info: PairInfo {
            contract_addr: env.contract.address.clone(),
            liquidity_token: Addr::unchecked(ADDR_WHILE_INSTANTIATION),
            asset_infos: msg.asset_infos.clone(),
            pair_type: PairType::Xyk {},
        },
        factory_addr: deps.api.addr_validate(msg.factory_addr.as_ref())?,
    };

    CONFIG.save(deps.storage, &config)?;

    let token_name = format_lp_token_name(msg.asset_infos, &deps.querier)?;

    // Create the LP token contract
    let instantiate_token_msg = SubMsg::reply_on_success(
        wasm_instantiate(
            msg.token_code_id,
            &TokenInstantiateMsg {
                name: token_name,
                symbol: "uLP".to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            },
            vec![],
            String::from("ZLL LP token"),
        )?,
        INSTANTIATE_TOKEN_REPLY_ID,
    );

    Ok(Response::new()
        .add_submessage(instantiate_token_msg)
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProvideLiquidity { assets, .. } => {
            assert_deposits_enabled(deps.branch())?;
            assert_balanced_assets_ratio(&assets)?;
            provide_liquidity(deps, env, info, assets)
        }
        ExecuteMsg::Receive(msg) => {
            assert_withrawals_enabled(deps.branch())?;
            receive_cw20(deps, env, info, msg)
        }
        _ => Ok(Response::default()),
    }
}

fn assert_deposits_enabled(deps: DepsMut) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let MarketPhaseResponse { phase } = deps.querier.query_wasm_smart(
        config.factory_addr,
        &cw_zll_std_market::msg::QueryMsg::GetMarketPhase {},
    )?;

    if !phase.can_lp_accept_deposits() {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

fn assert_balanced_assets_ratio(_assets: &[Asset; 2]) -> Result<(), ContractError> {
    // TODO: use pricing provided on the market's config level
    // let asset_a = MarketAsset::new(assets[0].amount, 1_000_000u128, 6);
    // let asset_b = MarketAsset::new(assets[1].amount, 1_000_000u128, 6);

    // println!("{:?} {:?}", &asset_a, &asset_b);

    // MarketLiquidity::new(asset_a, asset_b).is_balanced()?;

    Ok(())
}

fn assert_withrawals_enabled(deps: DepsMut) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let MarketPhaseResponse { phase } = deps.querier.query_wasm_smart(
        config.factory_addr,
        &cw_zll_std_market::msg::QueryMsg::GetMarketPhase {},
    )?;

    if !phase.can_lp_accept_withdrawals() {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// ## Description
/// Provides liquidity in the pair with the specified input parameters.
/// Returns a [`ContractError`] on failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **assets** is an array with two objects of type [`Asset`]. These are the assets available in the pool.
///
// NOTE - the address that wants to provide liquidity should approve the pair contract to pull its relevant tokens.
pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: [Asset; 2],
) -> Result<Response, ContractError> {
    assets[0].info.check(deps.api)?;
    assets[1].info.check(deps.api)?;

    for asset in assets.iter() {
        asset.assert_sent_native_token_balance(&info)?;
    }

    let config: Config = CONFIG.load(deps.storage)?;
    let mut pools: [Asset; 2] = config
        .pair_info
        .query_pools(&deps.querier, env.contract.address.clone())?;
    let deposits: [Uint128; 2] = [
        assets
            .iter()
            .find(|a| a.info.equal(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        assets
            .iter()
            .find(|a| a.info.equal(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    if deposits[0].is_zero() || deposits[1].is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    for (i, pool) in pools.iter_mut().enumerate() {
        // If the asset is a token contract, then we need to execute a TransferFrom msg to receive assets
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            messages.push(
                wasm_execute(
                    contract_addr,
                    &Cw20ExecuteMsg::TransferFrom {
                        owner: info.sender.to_string(),
                        recipient: env.contract.address.to_string(),
                        amount: deposits[i],
                    },
                    vec![],
                )?
                .into(),
            );
        } else {
            // If the asset is native token, the pool balance is already increased
            // To calculate the total amount of deposits properly, we should subtract the user deposit from the pool
            pool.amount = pool.amount.checked_sub(deposits[i])?;
        }
    }

    let total_share = query_supply(&deps.querier, config.pair_info.liquidity_token.clone())?;
    let share = if total_share.is_zero() {
        // Initial share = collateral amount
        Uint128::new(
            (U256::from(deposits[0].u128()) * U256::from(deposits[1].u128()))
                .integer_sqrt()
                .as_u128(),
        )
    } else {
        // min(1, 2)
        // 1. deposit_0 * (total_share / sqrt(pool_0 * pool_1))
        // == deposit_0 * total_share / pool_0
        // 2. deposit_1 * (total_share / sqrt(pool_1 * pool_1))
        // == deposit_1 * total_share / pool_1
        std::cmp::min(
            deposits[0].multiply_ratio(total_share, pools[0].amount),
            deposits[1].multiply_ratio(total_share, pools[1].amount),
        )
    };

    // Mint LP tokens for the sender
    messages.extend(mint_liquidity_token_message(
        &config,
        info.sender.clone(),
        share,
    )?);

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "provide_liquidity"),
        attr("sender", info.sender.as_str()),
        attr("receiver", info.sender.as_str()),
        attr("assets", format!("{}, {}", assets[0], assets[1])),
        attr("share", share.to_string()),
    ]))
}

/// ## Description
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise it returns the [`Response`] with the specified attributes if the operation was successful.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **cw20_msg** is an object of type [`Cw20ReceiveMsg`]. This is the CW20 message that has to be processed.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::WithdrawLiquidity {}) => withdraw_liquidity(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
        ),
        Ok(inner_msg) => Err(ContractError::CustomError {
            val: format!("Msg `{:?}` cannot be handled", inner_msg),
        }),
        Err(err) => Err(ContractError::Std(err)),
    }
}

/// ## Description
/// Withdraw liquidity from the pool. Returns a [`ContractError`] on failure,
/// otherwise returns a [`Response`] with the specified attributes if the operation was successful.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **sender** is an object of type [`Addr`]. This is the address that will receive assets back from the pair contract.
///
/// * **amount** is an object of type [`Uint128`]. This is the amount of LP tokens to burn.
pub fn withdraw_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage).unwrap();

    if info.sender != config.pair_info.liquidity_token {
        return Err(ContractError::Unauthorized {});
    }

    let (pools, total_share) = pool_info(deps.as_ref(), config.clone())?;
    let refund_assets = get_share_in_assets(&pools, amount, total_share);

    // Update the pool info
    let messages: Vec<CosmosMsg> = vec![
        refund_assets[0]
            .clone()
            .into_msg(&deps.querier, sender.clone())?,
        refund_assets[1]
            .clone()
            .into_msg(&deps.querier, sender.clone())?,
        wasm_execute(
            config.pair_info.liquidity_token.to_string(),
            &Cw20ExecuteMsg::Burn { amount },
            vec![],
        )?
        .into(),
    ];

    let attributes = vec![
        attr("action", "withdraw_liquidity"),
        attr("sender", sender.as_str()),
        attr("withdrawn_share", &amount.to_string()),
        attr(
            "refund_assets",
            format!("{}, {}", refund_assets[0], refund_assets[1]),
        ),
    ];

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(attributes))
}

/// ## Description
/// Returns the amount of pool assets that correspond to an amount of LP tokens.
/// ## Params
/// * **pools** are an array of [`Asset`] type items. These are the assets in the pool.
///
/// * **amount** is an object of type [`Uint128`]. This is the amount of LP tokens to compute a corresponding amount of assets for.
///
/// * **total_share** is an object of type [`Uint128`]. This is the total amount of LP tokens currently minted.
pub fn get_share_in_assets(
    pools: &[Asset; 2],
    amount: Uint128,
    total_share: Uint128,
) -> Vec<Asset> {
    let mut share_ratio = Decimal::zero();
    if !total_share.is_zero() {
        share_ratio = Decimal::from_ratio(amount, total_share);
    }

    pools
        .iter()
        .map(|a| Asset {
            info: a.info.clone(),
            amount: a.amount * share_ratio,
        })
        .collect()
}

fn mint_liquidity_token_message(
    config: &Config,
    recipient: Addr,
    amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let lp_token = config.pair_info.liquidity_token.clone();

    Ok(vec![wasm_execute(
        lp_token,
        &Cw20ExecuteMsg::Mint {
            recipient: recipient.to_string(),
            amount,
        },
        vec![],
    )?
    .into()])
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_binary(&query_pair_info(deps)?),
        QueryMsg::Pool {} => to_binary(&query_pool(deps)?),
        _ => Err(StdError::NotFound {
            kind: format!("Message {:?} cannot be handled", &msg),
        }),
    }
}

/// ## Description
/// Returns information about the pair contract in an object of type [`PairInfo`].
/// ## Params
/// * **deps** is an object of type [`Deps`].
pub fn query_pair_info(deps: Deps) -> StdResult<PairInfo> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config.pair_info)
}

/// ## Description
/// Returns the amounts of assets in the pair contract as well as the amount of LP
/// tokens currently minted in an object of type [`PoolResponse`].
/// ## Params
/// * **deps** is an object of type [`Deps`].
pub fn query_pool(deps: Deps) -> StdResult<PoolResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let (assets, total_share) = pool_info(deps, config)?;

    let resp = PoolResponse {
        assets,
        total_share,
    };

    Ok(resp)
}

/// ## Description
/// Returns the total amount of assets in the pool as well as the total amount of LP tokens currently minted.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **config** is an object of type [`Config`].
pub fn pool_info(deps: Deps, config: Config) -> StdResult<([Asset; 2], Uint128)> {
    let contract_addr = config.pair_info.contract_addr.clone();
    let pools: [Asset; 2] = config.pair_info.query_pools(&deps.querier, contract_addr)?;
    let total_share: Uint128 = query_supply(&deps.querier, config.pair_info.liquidity_token)?;

    Ok((pools, total_share))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.result.is_err() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: msg.result.unwrap_err(),
        }));
    }

    match msg.id {
        INSTANTIATE_TOKEN_REPLY_ID => reply_on_instantiate_token(deps, env, msg),
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("reply id `{:?}` is invalid", msg.id),
        })),
    }
}

fn reply_on_instantiate_token(
    deps: DepsMut,
    _env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.pair_info.liquidity_token != Addr::unchecked(ADDR_WHILE_INSTANTIATION) {
        return Err(ContractError::Unauthorized {});
    }

    let res: MsgInstantiateContractResponse = parse_reply_instantiate_data(msg)?;

    config.pair_info.liquidity_token = deps.api.addr_validate(&res.contract_address)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("liquidity_token_addr", config.pair_info.liquidity_token))
}
