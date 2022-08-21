use cosmwasm_std::{Addr, StdResult};
use cw_multi_test::{App, ContractWrapper};
use cw_zll_std_market::response::{MarketPhase, MarketPhaseResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::contract_helpers::ContractBase;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketContract(pub Addr);

impl MarketContract {
    pub fn get_market_phase(
        &self,
        app: &App<cosmwasm_std::Empty, terra_cosmwasm::TerraQueryWrapper>,
    ) -> StdResult<MarketPhase> {
        let response: MarketPhaseResponse = app.wrap().query_wasm_smart(
            self.addr(),
            &cw_zll_std_market::msg::QueryMsg::GetMarketPhase {},
        )?;

        Ok(response.phase)
    }
}

impl ContractBase for MarketContract {
    type ExecuteMsg = cw_zll_std_market::msg::ExecuteMsg;

    fn addr(&self) -> Addr {
        self.0.clone()
    }

    fn contract_code() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(
            cw_zll_market::contract::execute,
            cw_zll_market::contract::instantiate,
            cw_zll_market::contract::query,
        )
        .with_reply(cw_zll_market::contract::reply);

        Box::new(contract)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidityPoolContract(pub Addr, pub Addr);

impl LiquidityPoolContract {
    pub fn lp_token_contract(&self) -> LiquidityPoolTokenContract {
        LiquidityPoolTokenContract(self.1.clone())
    }
}

impl ContractBase for LiquidityPoolContract {
    type ExecuteMsg = cw_zll_std_liquidity_pool::msg::ExecuteMsg;

    fn addr(&self) -> Addr {
        self.0.clone()
    }

    fn contract_code() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(
            cw_zll_liquidity_pool::contract::execute,
            cw_zll_liquidity_pool::contract::instantiate,
            cw_zll_liquidity_pool::contract::query,
        )
        .with_reply(cw_zll_liquidity_pool::contract::reply);

        Box::new(contract)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw20TokenContract(pub Addr);

impl ContractBase for Cw20TokenContract {
    type ExecuteMsg = cw20_base::msg::ExecuteMsg;

    fn addr(&self) -> Addr {
        self.0.clone()
    }

    fn contract_code() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );

        Box::new(contract)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidityPoolTokenContract(pub Addr);

impl ContractBase for LiquidityPoolTokenContract {
    type ExecuteMsg = cw_zll_liquidity_pool_token::msg::ExecuteMsg;

    fn addr(&self) -> Addr {
        self.0.clone()
    }

    fn contract_code() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(
            cw_zll_liquidity_pool_token::contract::execute,
            cw_zll_liquidity_pool_token::contract::instantiate,
            cw_zll_liquidity_pool_token::contract::query,
        );

        Box::new(contract)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OptionTokenContract(pub Addr);

impl ContractBase for OptionTokenContract {
    type ExecuteMsg = cw_zll_option_token::msg::ExecuteMsg;

    fn addr(&self) -> Addr {
        self.0.clone()
    }

    fn contract_code() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(
            cw_zll_option_token::contract::execute,
            cw_zll_option_token::contract::instantiate,
            cw_zll_option_token::contract::query,
        );

        Box::new(contract)
    }
}
