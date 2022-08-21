use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_zll_std_liquidity_pool::ap::asset::PairInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure stores the main config parameters for a constant product pair contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// General pair information (e.g pair type)
    pub pair_info: PairInfo,
    /// Market contract address
    pub factory_addr: Addr,
}

/// ## Description
/// Stores the config struct at the given key
pub const CONFIG: Item<Config> = Item::new("config");
