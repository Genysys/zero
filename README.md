# Zero-Liquidation Loans protocol: Minimal Viable Product

## Executive summary

This document describes a Minimal Viable Product (MVP) for Zero-Liquidation Loans (ZLL) protocol.

A Liquidity Pool is the core component of the Automated Market Maker. Initially, in the _Liquidity Pooling Phase_, it needs liquidity from Liquidity Providers (they can withdraw their assets back only in the _Post-Settlement Phase_).

Once enough liquidity is available, the protocol switches to the next phase: _Automated Market Maker_. During this phase, Borrowers and Lenders are able to use the Liquidty Pool.

A Borrower can interact with a Liquidity Pool and borrow a repayment asset _RA_ while providing a collateral in the deposit asset _DA_. The Borrower buys a CALL option _C<sub>K</sub>_ with a strike price _K_ from the Liquidity Pool. It lets the Borrower (during the _Settlement Phase_) to either buy the collateral back, or just walk away and effectively pay the loan back to the Liquidity Pool with the collateral. 

A Lender  can interact with the Liquidity Pool and lend a repayment asset _RA_ while reserving collateral in the deposit asset _DA_. The Lender sells a PUT option _P<sub>K</sub>_ with a strike price _K_ to Liquidity Pool . It lets the Liquidity Pool operator to either sell the repayment asset back to the Lender with a strike price _K_ (during the _Settlement Phase_), or walk away and pay the Lender back with the collateral. In the latter case, the Lender (during the _Post-Settlement Phase_) claims the collateral back from the Liquidity Pool.

## Documentation

- [Protocol specification](./docs/protocol-spec.md)
- [Protocol implemenation](./docs/protocol-impl-design.md)