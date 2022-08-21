# Protocol specification

## Diagram

A sequence diagram showing available interactions among actors in different phases of a particular market.

```mermaid

sequenceDiagram
    participant LP as Liquidity Provider
    participant M as Market (+Liquidity Pool)
    participant B as Borrower
    participant L as Lender

    Note over LP,L: 1st Phase: Pooling liquidity
    LP->>M: Provide collateral asset `CA` and lending asset `LA`
    M-->>LP: Issue LP shares

    Note over LP,L: 2nd Phase: Automated Market Maker
    par Borrow

        B->>M: Deposit collateral asset `CA`
        M-->>B: Write CALL option `C_{K}` with strike price of `K` and provide loan `K - P_{K}` in `LA` asset

    and Lend

        L->>M: Deposit lending asset `LA`
        L->>M: Write PUT option `P_{K}`
        M-->>M: Lock `CA` asset as collateral for the loan (it's payment for the `P_{K}` option)

    end

    Note over LP,L: 3rd Phase: Settlement
    par Borrower decides what to do with their CALL option

        alt Collateral in `CA` asset is worth more than the strike price `K` in `LA` asset

            B->>B: Execute option `C_{K}`: Buy `CA` asset

            B->>M: Repay the loan `K` in the `LA` asset
            M-->>B: Return the deposit in `CA` asset

        else Walk away from the loan

            B->>B: Resign from deposit in `CA` asset and keep `K - P_{K}` in `LA` asset

        end

    and Market (owner) decides what to do with their PUT option

        alt Strike price `K` for Locked `CA` asset as collateral is worth more than the lent `LA` asset

            M->>M: Execute option `P_{K}`: Sell `LA` asset

            M->>L: Send the initially lent `LA` asset + interest
            M-->>M: Unlock `CA` asset and return it to pool

        else Walk away from the loan

            Note over LP,L: 4th Phase: Post-Settlement
            L->>M: Claim locked collateral in `CA` asset
            M-->>L: Unlock&send collateral in `CA` asset

        end

        LP->>M: Burns LP shares
        M-->>LP: Sends pro-rated amount of `CA` and `LA`
        
    end
```

## Roles

### Borrower

Deposits collateral asset into the liquidity pool to receive a zero-liquidation loan and an option to reclaim the collateral asset for a pre-agreed repayment amount once the option expired.

### Lender

Deposit lent asset into the liquidity pool and gives it the option for a pre-agreed repayment amount in the lent asset (including interest). Alternatively, after the option expired, can claim pre-agreed amount of the collateral asset as a repayment.

### Liquidity Provider

Provides liquidity to the liquidity pool (in both, the collateral asset and the lent asset) and receives pool shares back. Can use shares to withdraw their liquidity 

### Liquidity Pool

Enables Borrowers to borrow, Lenders to lend, and Liquidity Providers to provide liquidity.

Can earn fees from borrowing assets, and pay interest for lending assest. Remaining revenue is split between pool shareholders.

### Liqudity Pool Operator

Maintains the Liqudity Pool. They take care of setting up correct configuration, and having it updated while changing market conditions.

They can also exectute PUT options which were written by Lenders.

## Liquidity Pool phases

### 1. Pooling liquidity

Liquidity providers can deposit assets into the pool and receive pool shares in return.

No borrowing, nor lending is possible yet.

### 2. Automated Market Maker

Once sufficient liquidity is provided, the pool enters the active phase.
It lets Borrowers borrow assets, and Lenders lend assets.

### 3. Settlement

All borrowing, and lending has ended by this phase.

Borrowers can execute their CALL options repay their loans.

The Liqudity Pool Operator decides which PUT options are worth executing, and executes them accordingly.

### 4. Post-settlement

Lenders who written PUT options that didn't get execututed can now claim the collateral asset from the Liquidity Pool.

Liquidity Providers can now burn their pool shares in order to withdraw remaining liquidity from the Liquidity Pool.
