use std::{convert::TryInto, ops::Mul, str::FromStr};

use cosmwasm_std::{Decimal256, DivideByZeroError, OverflowError, Uint128, Uint256};

pub trait LiquidityAssetRatio {
    /// Calculates what is the amount of the asset that would be in balance with the compared asset
    fn balance_with(&self, rhs: &Self) -> Result<Uint128, MarketLiquidityError> {
        let decimals_diff = self.decimals() as i8 - rhs.decimals() as i8;

        let adjusted_base = if decimals_diff.is_positive() {
            rhs.amount()
                .checked_mul(Uint128::new(10).checked_pow(decimals_diff as u32)?)?
        } else if decimals_diff.is_negative() {
            rhs.amount()
                .checked_div(Uint128::new(10).checked_pow(decimals_diff.abs() as u32)?)?
        } else {
            rhs.amount()
        };

        Ok(adjusted_base.multiply_ratio(rhs.base_price(), self.base_price()))
    }

    /// Amount of currency in minor (atomic) unit
    fn amount(&self) -> Uint128;

    /// Asset's price in base currency
    fn base_price(&self) -> Uint128;

    /// Decimal places used
    fn decimals(&self) -> u8;
}

pub trait MarketAssetLiquidity<TCurrency: LiquidityAssetRatio> {
    fn get_borrow_asset(&self) -> TCurrency;

    fn get_collateral_asset(&self) -> TCurrency;
}

pub trait MarketAssetLiquidityRatio<TCurrency: LiquidityAssetRatio>:
    MarketAssetLiquidity<TCurrency>
{
    fn get_lp_borrow_asset_amount(
        &self,
        collateral_asset_amount: impl Into<TCurrency>,
    ) -> Result<Uint128, MarketLiquidityError> {
        self.get_borrow_asset()
            .balance_with(&collateral_asset_amount.into())
    }

    fn get_lp_collateral_asset_amount(
        &self,
        borrow_asset_amount: impl Into<TCurrency>,
    ) -> Result<Uint128, MarketLiquidityError> {
        self.get_collateral_asset()
            .balance_with(&borrow_asset_amount.into())
    }
}

/// A single asset that is included on the market (within a liquidity pool).
/// Can be either asset to borrow, or asset to lend.
///
/// Its amount can be expressed in minor (atomic) or major units.
/// - The minor unit is the smallest possible amount of the asset (determined by decimals)
/// - The major unit is the amount of 10^decimals in the minor unit.
///
/// Example:
/// ```rs
/// let decimals: u8 = 6;
/// let minor_amount: Uint128 = Uint::new(1);
/// let major_amount: Uint128 = Uint::new(1_000_000);
/// ```
#[derive(Debug)]
pub struct MarketAsset {
    /// A price in the base market currency (for example, USD)
    base_price: Uint128,

    /// Decimal places required for a major unit
    decimals: u8,

    /// Amount in the minor (atomic) unit
    amount: Uint128,
}

impl MarketAsset {
    pub fn new(amount: impl Into<Uint128>, base_price: impl Into<Uint128>, decimals: u8) -> Self {
        Self {
            amount: amount.into(),
            base_price: base_price.into(),
            decimals,
        }
    }

    pub fn parse_base_price(base_price_str: &str, decimals: u8) -> Uint128 {
        match Decimal256::from_str(base_price_str) {
            Ok(price) => {
                let price_adjusted =
                    price.mul(Uint256::from_uint128(Uint128::new(10)).pow(decimals as u32));

                match price_adjusted.try_into() {
                    Ok(price) => price,
                    Err(_) => Uint128::zero(),
                }
            }
            Err(_) => Uint128::zero(),
        }
    }

    pub fn set_amount(&mut self, amount: impl Into<Uint128>) {
        self.amount = amount.into();
    }
}

impl Default for MarketAsset {
    fn default() -> Self {
        let decimals = 3u8;

        Self::new(
            Uint128::default(),
            Uint128::new(1).pow(decimals as u32),
            decimals,
        )
    }
}

impl LiquidityAssetRatio for MarketAsset {
    fn amount(&self) -> Uint128 {
        self.amount
    }

    fn base_price(&self) -> Uint128 {
        self.base_price
    }

    fn decimals(&self) -> u8 {
        self.decimals
    }
}

#[derive(Debug)]
pub struct MarketLiquidity {
    borrow_asset: MarketAsset,
    collateral_asset: MarketAsset,
}

impl MarketLiquidity {
    pub fn new(borrow_asset: MarketAsset, collateral_asset: MarketAsset) -> Self {
        Self {
            borrow_asset,
            collateral_asset,
        }
    }

    pub fn is_balanced(&self) -> Result<(), MarketLiquidityError> {
        let expected_borrow_asset_amount =
            self.borrow_asset.balance_with(&self.collateral_asset)?;
        let expected_collateral_asset_amount =
            self.collateral_asset.balance_with(&self.borrow_asset)?;

        // let's check if enough borrowed asset was provided
        if expected_borrow_asset_amount > self.borrow_asset.amount() {
            return Err(MarketLiquidityError::borrow_asset_imbalance(
                self.borrow_asset.amount(),
                expected_borrow_asset_amount,
            ));
        }

        // let's check if enough collateral asset was provided
        if expected_collateral_asset_amount > self.collateral_asset.amount() {
            return Err(MarketLiquidityError::collateral_asset_imbalance(
                self.collateral_asset.amount(),
                expected_collateral_asset_amount,
            ));
        }

        Ok(())
    }
}

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MarketLiquidityError {
    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error(
        "Borrow asset imbalance: provided = {provided_balance:?}; expected = {expected_balance:?}"
    )]
    BorrowAssetImbalance {
        expected_balance: Uint128,
        provided_balance: Uint128,
    },

    #[error("Collateral asset imbalance: provided = {provided_balance:?}; expected = {expected_balance:?}")]
    CollateralAssetImbalance {
        expected_balance: Uint128,
        provided_balance: Uint128,
    },
}

impl MarketLiquidityError {
    pub fn borrow_asset_imbalance(provided_balance: Uint128, expected_balance: Uint128) -> Self {
        Self::BorrowAssetImbalance {
            expected_balance,
            provided_balance,
        }
    }

    pub fn collateral_asset_imbalance(
        provided_balance: Uint128,
        expected_balance: Uint128,
    ) -> Self {
        Self::CollateralAssetImbalance {
            expected_balance,
            provided_balance,
        }
    }
}

#[cfg(test)]
mod tests {
    mod parsing_base_price {
        use cosmwasm_std::Uint128;

        use crate::liquidity::MarketAsset;

        #[test]
        fn parses_basic_price_correctly_when_price_is_very_small() {
            let basic_price_str = "0.00000006";
            let decimals = 8u8;

            assert_eq!(
                MarketAsset::parse_base_price(basic_price_str, decimals,),
                Uint128::new(6),
            )
        }

        #[test]
        fn parses_basic_price_correctly_when_price_is_quite_small() {
            let basic_price_str = "0.0003212";
            let decimals = 8u8;

            assert_eq!(
                MarketAsset::parse_base_price(basic_price_str, decimals,),
                Uint128::new(32_120),
            )
        }

        #[test]
        fn parses_basic_price_correctly_when_price_is_small() {
            let basic_price_str = "0.09832422";
            let decimals = 8u8;

            assert_eq!(
                MarketAsset::parse_base_price(basic_price_str, decimals,),
                Uint128::new(9_832_422),
            )
        }

        #[test]
        fn parses_basic_price_correctly_when_price_is_medium() {
            let basic_price_str = "1.0";
            let decimals = 8u8;

            assert_eq!(
                MarketAsset::parse_base_price(basic_price_str, decimals,),
                Uint128::new(100_000_000),
            )
        }

        #[test]
        fn parses_basic_price_correctly_when_price_is_large() {
            let basic_price_str = "698.123512";
            let decimals = 8u8;

            assert_eq!(
                MarketAsset::parse_base_price(basic_price_str, decimals,),
                Uint128::new(69_812_351_200),
            )
        }

        #[test]
        fn parses_basic_price_correctly_when_price_is_very_large() {
            let basic_price_str = "329698.123512";
            let decimals = 8u8;

            assert_eq!(
                MarketAsset::parse_base_price(basic_price_str, decimals,),
                Uint128::new(32_969_812_351_200),
            )
        }
    }

    mod checking_provided_assets_balance {
        use cosmwasm_std::Uint128;

        use crate::liquidity::{
            LiquidityAssetRatio, MarketAsset, MarketLiquidity, MarketLiquidityError,
        };

        #[test]
        fn calculactes_correclty_balanced_market_assets_amounts() {
            // example: 2000 UST (1.0 in USD)
            let borrow_asset = MarketAsset::new(
                2_000_000_000u128,
                MarketAsset::parse_base_price("1.0", 6),
                6,
            );

            // example: 10 Custom Token (95.53 in USD)
            let collateral_asset = MarketAsset::new(
                10_000_000_000u128,
                MarketAsset::parse_base_price("95.53", 9),
                9,
            );

            let expected_borrow_asset_amount =
                borrow_asset.balance_with(&collateral_asset).unwrap();

            assert_eq!(expected_borrow_asset_amount, Uint128::new(955_300_000_000));

            let expected_collateral_asset_amount =
                collateral_asset.balance_with(&borrow_asset).unwrap();

            assert_eq!(expected_collateral_asset_amount, Uint128::new(20_935_831))
        }

        #[test]
        fn allows_correctly_balanced_liquidity() {
            // USD equivalent of 10,000.00
            let total_deposit_in_usd = 10_000_000_000u128;
            let equivalent_deposit_in_usd_asset = MarketAsset::new(
                total_deposit_in_usd * 5 / 10,
                MarketAsset::parse_base_price("1.0", 6),
                6,
            );

            // stable coin (1.0 in USD)
            let mut borrow_asset =
                MarketAsset::new(u128::default(), MarketAsset::parse_base_price("1.0", 6), 6);

            borrow_asset.set_amount(
                borrow_asset
                    .balance_with(&equivalent_deposit_in_usd_asset)
                    .unwrap(),
            );

            // Custom Token (95.53 in USD)
            let mut collateral_asset = MarketAsset::new(
                u128::default(),
                MarketAsset::parse_base_price("95.53", 6),
                6,
            );

            collateral_asset.set_amount(collateral_asset.balance_with(&borrow_asset).unwrap());

            let market_liquidity = MarketLiquidity::new(borrow_asset, collateral_asset);

            assert_eq!(market_liquidity.is_balanced().is_ok(), true)
        }

        #[test]
        fn prevents_incorrectly_balanced_liquidity_when_not_enough_collateral_asset_provided() {
            // USD equivalent of 10,000.00
            let total_deposit_in_usd = 10_000_000_000u128;
            let equivalent_deposit_in_usd_asset = MarketAsset::new(
                total_deposit_in_usd * 5 / 10,
                MarketAsset::parse_base_price("1.0", 6),
                6,
            );

            // stable coin (1.0 in USD)
            let mut borrow_asset =
                MarketAsset::new(u128::default(), MarketAsset::parse_base_price("1.0", 6), 6);

            borrow_asset.set_amount(
                borrow_asset
                    .balance_with(&equivalent_deposit_in_usd_asset)
                    .unwrap(),
            );

            // Custom Token (95.53 in USD)
            let mut collateral_asset = MarketAsset::new(
                u128::default(),
                MarketAsset::parse_base_price("95.53", 6),
                6,
            );

            let expected_collateral_asset_amount =
                collateral_asset.balance_with(&borrow_asset).unwrap();

            let actual_collateral_asset_amount = expected_collateral_asset_amount
                .clone()
                // let's lower provided asset amount
                .checked_sub(Uint128::new(1u128))
                .unwrap();

            collateral_asset.set_amount(actual_collateral_asset_amount);

            let market_liquidity = MarketLiquidity::new(borrow_asset, collateral_asset);

            assert_eq!(
                market_liquidity.is_balanced().unwrap_err(),
                MarketLiquidityError::collateral_asset_imbalance(
                    actual_collateral_asset_amount,
                    expected_collateral_asset_amount
                )
            );
        }
        #[test]
        fn prevents_incorrectly_balanced_liquidity_when_not_enough_borrow_asset_provided() {
            // USD equivalent of 10,000.00
            let total_deposit_in_usd = 10_000_000_000u128;
            let equivalent_deposit_in_usd_asset = MarketAsset::new(
                total_deposit_in_usd * 5 / 10,
                MarketAsset::parse_base_price("1.0", 6),
                6,
            );

            // Custom Token (95.53 in USD)
            let mut collateral_asset = MarketAsset::new(
                u128::default(),
                MarketAsset::parse_base_price("95.53", 6),
                6,
            );

            collateral_asset.set_amount(
                collateral_asset
                    .balance_with(&equivalent_deposit_in_usd_asset)
                    .unwrap(),
            );

            // stable coin (1.0 in USD)
            let mut borrow_asset =
                MarketAsset::new(u128::default(), MarketAsset::parse_base_price("1.0", 6), 6);

            let expected_borrow_asset_amount =
                borrow_asset.balance_with(&collateral_asset).unwrap();

            let actual_borrow_asset_amount = expected_borrow_asset_amount
                .clone()
                // let's lower provided asset amount
                .checked_sub(Uint128::new(1u128))
                .unwrap();

            borrow_asset.set_amount(actual_borrow_asset_amount);

            let market_liquidity = MarketLiquidity::new(borrow_asset, collateral_asset);

            println!("{:?}", &market_liquidity.is_balanced());

            assert_eq!(
                market_liquidity.is_balanced().unwrap_err(),
                MarketLiquidityError::borrow_asset_imbalance(
                    actual_borrow_asset_amount,
                    expected_borrow_asset_amount
                )
            );
        }
    }
}
