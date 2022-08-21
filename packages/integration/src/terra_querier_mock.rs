use anyhow::Result as AnyResult;
use cosmwasm_std::{to_binary, Addr, Decimal, Empty, Uint128};
use cw_multi_test::CustomHandler;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper};

pub struct TerraCustomQueryHandler;

impl CustomHandler<Empty, TerraQueryWrapper> for TerraCustomQueryHandler {
    fn execute(
        &self,
        _api: &dyn cosmwasm_std::Api,
        _storage: &mut dyn cosmwasm_std::Storage,
        _block: &cosmwasm_std::BlockInfo,
        _sender: Addr,
        _msg: Empty,
    ) -> AnyResult<cw_multi_test::AppResponse> {
        todo!()
    }

    fn query(
        &self,
        _api: &dyn cosmwasm_std::Api,
        _storage: &dyn cosmwasm_std::Storage,
        _block: &cosmwasm_std::BlockInfo,
        msg: TerraQueryWrapper,
    ) -> AnyResult<cosmwasm_std::Binary> {
        match msg.query_data {
            TerraQuery::TaxRate {} => Ok(to_binary(&TaxRateResponse {
                rate: Decimal::zero(),
            })?),
            TerraQuery::TaxCap { denom: _ } => Ok(to_binary(&TaxCapResponse {
                cap: Uint128::zero(),
            })?),
            _ => todo!(),
        }
    }
}
