use cosmwasm_std::Uint128;

pub struct ExpiryTime {
    pub time_to_expiry: Uint128,
    pub sqrt_time_to_expiry: Uint128,
}
