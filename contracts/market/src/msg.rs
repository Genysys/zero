use cosmwasm_std::Addr;
use cw_zll_std_liquidity_pool::ap::{AssetInfo, Asset};
use cw_zll_std_market::state::MarketPhasesInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        provided_deposit: Asset,
    },
}
