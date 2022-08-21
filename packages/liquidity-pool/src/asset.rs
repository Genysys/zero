use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::Addr;

pub fn create_coin_asset(amount: u128, denom: &str) -> Asset {
    Asset {
        info: AssetInfo::NativeToken {
            denom: denom.into(),
        },
        amount: amount.into(),
    }
}

pub fn create_token_asset(amount: u128, contract_addr: Addr) -> Asset {
    Asset {
        info: AssetInfo::Token { contract_addr },
        amount: amount.into(),
    }
}
