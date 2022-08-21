use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::MarketPhasesInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarketOperatorResponse {
    pub market_operator: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LiquidityPoolResponse {
    pub liquidity_pool: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarketPhaseResponse {
    pub phase: MarketPhase,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketPhase {
    ProvidingLiquidity,
    AutomatedMarketMaker,
    Settlement,
    PostSettlement,
}

impl MarketPhase {
    pub fn can_lp_accept_deposits(self) -> bool {
        self == Self::ProvidingLiquidity
    }

    pub fn can_lp_accept_withdrawals(self) -> bool {
        self == Self::PostSettlement
    }

    pub fn can_amm_accept_borrowing(self) -> bool {
        self == Self::AutomatedMarketMaker
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MarketPhasesInfoResponse {
    pub market_started_at: u64,
    pub lp_phase_ends_at: u64,
    pub amm_phase_ends_at: u64,
    pub settlement_phase_ends_at: u64,
}

impl From<MarketPhasesInfo> for MarketPhasesInfoResponse {
    fn from(market_phases_info: MarketPhasesInfo) -> Self {
        Self {
            market_started_at: market_phases_info.market_started_at,
            lp_phase_ends_at: market_phases_info.lp_phase_ends_at,
            amm_phase_ends_at: market_phases_info.amm_phase_ends_at,
            settlement_phase_ends_at: market_phases_info.settlement_phase_ends_at,
        }
    }
}
