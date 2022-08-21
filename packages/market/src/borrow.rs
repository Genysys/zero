use cw_zll_std_liquidity_pool::ap::Asset;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BorrowingTermsResponse {
    pub borrow: Asset,
    pub interest: Asset,
    pub repayment: Asset,
}

pub struct BorrowingTerms {
    pub borrow: Asset,
    pub interest: Asset,
    pub repayment: Asset,
}

impl From<BorrowingTerms> for BorrowingTermsResponse {
    fn from(borrowing_terms: BorrowingTerms) -> Self {
        Self {
            borrow: borrowing_terms.borrow,
            interest: borrowing_terms.interest,
            repayment: borrowing_terms.repayment,
        }
    }
}
