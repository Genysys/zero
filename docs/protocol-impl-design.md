# Protocol implemenation

## Public interface design

```mermaid
flowchart LR
    subgraph Actors
        Borrower
        Lender
        MarketOperator[Market Operator]
        LiqudityProvider[Liqudity Provider]
    end

    subgraph Contracts
        subgraph LP
            LP_deposit[deposit]
            LP_withdraw[withdraw]
        end

        subgraph M[Market]
            M_get_borrowing_terms[get borrowing terms]
            M_get_lending_terms[get lending terms]
            M_borrow[borrow]
            M_lend[lend]
            M_execute_call_option[repay borrower's loan]
            M_execute_put_option[repay market's loan]
            M_adjust_put_price[adjust PUT option pricing]
            M_claim_collateral[claim collateral]
        end
    end

    Borrower --> |2nd Phase| M_borrow & M_get_borrowing_terms
    Borrower --> |3rd Phase| M_execute_call_option

    Lender --> |2nd Phase| M_lend & M_get_lending_terms
    Lender --> |4th Phase| M_claim_collateral

    MarketOperator --> |3rd Phase| M_execute_put_option

    MarketOperator --> |1st Phase or 2nd Phase| M_adjust_put_price

    LiqudityProvider --> |1st Phase| LP_deposit
    LiqudityProvider --> |4th Phase|LP_withdraw
```

## Non-public (internal) interface design

Requirements applied:

1. The Market contract is deployed and managed by its Maintainer.
    1. Maintenance is about setting market parameters accordingly to current market conditions.
2. The Market contract is the owner of:
    1. the Liquidity Pool (LP) contract;
    2. the Option Token (OT) contract.
3. The Liquidity Pool contract:
    1. the custodian of all liquidity coming from Liqudity Providers, Borrowers, Lenders, Meta Pool;
    2. is the owner of the Liquidity Pool Token (LPT) contract;
    3. enables the Market contract to transfer assets in/out its custody, as required.
4. The Liquidity Pool Token contract is:
    1. held by Liquidity Providers and Meta Pool while they have their liquidity locked in the Liquidity Pool;
    2. used to withdraw pro-rated liquidity from the Liquidity Pool.
5. The Option Token contract is:
    1. held by either, a Borrower (CALL option), or Market Contract (PUT option);
    2. used to execute the option it describes.
6. The Meta Pool contract is deployed and managed by its Maintainer (MMP):
    1. Maintenance is about moving liquidity from and to Multiple Liquidy Pools;
7. The Meta Pool contract is:
    1. the custodian of all the liquidity coming from Liquidity Providers and held as different Liquidity Pools Tokens;
8. The Meta Pool Token contract is:
    1. held by Meta Pool's Liquidity Providers while they have their liquidity locked in the Meta Pool;
    2. used to withdraw pro-rated liquidity from the Meta Pool;


```mermaid

flowchart LR
    subgraph Contracts
    direction LR
        subgraph MetaPool [Meta Pool]
            MetaPool_deposit[deposit]
            
            MetaPool_MMP_provide_liquidity_to[MMP provide liquidity]
            MetaPool_MMP_move_liquidity_from[MMP move liquidity]

            MetaPool_withdraw[withdraw]
            
        end

        MetaPool_MMP_provide_liquidity_to ---> LP_deposit
        MetaPool_MMP_move_liquidity_from ---> LP_withdraw

        MetaPool_deposit ---> |issue shares| MPT_mint
        MetaPool_withdraw ---> |burn shares| MPT_burn

        subgraph MPT["Meta Pool Token (CW20)"]
            MPT_mint[mint]
            MPT_burn[burn]
        end
        
        subgraph LP
            LP_deposit[deposit]
            LP_withdraw[withdraw]
            LP_transfer_assets_and_update_supplies[transfer tokens & update supplies]
            LP_get_supply_info[get supply info]
        end

        subgraph LPT["LP Token (CW20)"]
            LPT_mint[mint]
            LPT_burn[burn]
        end

        LP_deposit ---> |issue shares| LPT_mint
        LP_withdraw ---> |burn shares| LPT_burn

        subgraph OT["Option Token (CW721)"]
            OT_mint[mint]
            OT_burn[burn]
        end

        subgraph M[Market]
            M_get_borrowing_terms[get borrowing terms]
            M_get_lending_terms[get lending terms]
            M_borrow[borrow]
            M_lend[lend]
            M_execute_call_option[repay borrower's loan]
            M_execute_put_option[repay market's loan]
            M_adjust_put_price[adjust PUT option pricing]
            M_get_put_price[get PUT option pricing]
            M_claim_collateral[claim collateral]
        end

        M_borrow ---> |"issue a CALL option for borrower"| OT_mint
        M_execute_call_option --> |"execute the CALL option of borrower"| OT_burn
        M_execute_put_option --> |"execute the PUT option of market"| OT_burn
        
        M_lend ---> |"issue a PUT option for market"| OT_mint

        M_borrow ----> LP_transfer_assets_and_update_supplies

        M_lend ----> LP_transfer_assets_and_update_supplies

        M_execute_call_option --> LP_transfer_assets_and_update_supplies

        M_execute_put_option --> LP_transfer_assets_and_update_supplies

        M_claim_collateral --> LP_transfer_assets_and_update_supplies

        M_get_borrowing_terms --> LP_get_supply_info & M_get_put_price

        M_adjust_put_price -.-> |provides| M_get_put_price
        M_get_lending_terms --> LP_get_supply_info & M_get_put_price
    
    end

```
