use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Empty, StdResult, WasmMsg};
use cw_multi_test::Contract;
use serde::Serialize;

pub trait ContractBase {
    type ExecuteMsg: Serialize;

    fn contract_code() -> Box<dyn Contract<Empty>>;

    fn addr(&self) -> Addr;

    fn call(&self, msg: &Self::ExecuteMsg, funds: Option<Vec<Coin>>) -> StdResult<CosmosMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg: to_binary(msg)?,
            funds: funds.unwrap_or_default(),
        }
        .into())
    }
}
