use cosmwasm_std::{Addr, StdResult, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub market_operator: Addr,
    pub liquidity_pool: Addr,
    pub blocks_per_year: u64,
    pub alpha: u64,
    pub market_phases_info: MarketPhasesInfo,
}

const CONFIG: Item<Config> = Item::new("config");

pub fn set_config(storage: &mut dyn Storage, config: Config) -> StdResult<()> {
    CONFIG.save(storage, &config)
}

pub fn set_market_operator(storage: &mut dyn Storage, market_operator: Addr) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| {
        config.market_operator = market_operator;
        Ok(config)
    })
}

pub fn get_market_operator(storage: &dyn Storage) -> StdResult<Addr> {
    let config = CONFIG.load(storage)?;

    Ok(config.market_operator)
}

pub fn set_liquidity_pool(storage: &mut dyn Storage, liquidity_pool: Addr) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| {
        config.liquidity_pool = liquidity_pool;
        Ok(config)
    })
}

pub fn get_liquidity_pool(storage: &dyn Storage) -> StdResult<Addr> {
    let config = CONFIG.load(storage)?;

    Ok(config.liquidity_pool)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketPhasesInfo {
    pub market_started_at: u64,
    pub lp_phase_ends_at: u64,
    pub amm_phase_ends_at: u64,
    pub settlement_phase_ends_at: u64,
}

pub fn set_market_info(
    storage: &mut dyn Storage,
    market_phases_info: MarketPhasesInfo,
) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| {
        config.market_phases_info = market_phases_info;
        Ok(config)
    })
}

pub fn get_market_info(storage: &dyn Storage) -> StdResult<MarketPhasesInfo> {
    let config = CONFIG.load(storage)?;

    Ok(config.market_phases_info)
}

pub fn get_blocks_per_year(storage: &dyn Storage) -> StdResult<u64> {
    let config = CONFIG.load(storage)?;

    Ok(config.blocks_per_year)
}

pub fn get_alpha(storage: &dyn Storage) -> StdResult<u64> {
    let config = CONFIG.load(storage)?;

    Ok(config.alpha)
}
