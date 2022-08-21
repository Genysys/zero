use cosmwasm_std::{OverflowError, StdError};
use cw_zll_std_liquidity_pool::liquidity::MarketLiquidityError;
use cw_zll_std_utils::reply::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    MarketLiquidityError(#[from] MarketLiquidityError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Doubling assets in asset infos")]
    DoublingAssets {},

    #[error("Asset mismatch between the requested and the stored asset in contract")]
    AssetMismatch {},

    #[error("Event of zero transfer")]
    InvalidZeroAmount {},
}
