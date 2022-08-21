use cosmwasm_std::Addr;
use cw_zll_std_liquidity_pool::ap::{Asset, AssetInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::MarketPhasesInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub market_operator: Addr,
    pub liquidity_pool_code_id: u64,
    pub liquidity_pool_token_code_id: u64,
    pub asset_infos: [AssetInfo; 2],
    pub market_phases_info: MarketPhasesInfo,
    pub blocks_per_year: u64,
    pub alpha: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Borrow {
        expected_borrow: Asset,
        pledged_collateral: Asset,
    },
}

/// This structure describes the query messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetMarketOperator {},
    GetLiquidityPool {},
    GetMarketPhase {},
    GetMarketPhasesInfo {},
    GetBorrowingTerms { pledged_collateral: Asset },
}
