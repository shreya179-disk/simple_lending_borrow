FORMULAS:
Users deposit tokens into a bank (or vault), and in return, they receive deposit shares representing their ownership.
The number of shares a user gets is based on the proportion of their deposit to the total bank deposits.

        Deposit Ratio= Amount Deposited / Total Bank Deposits 

This tells us what fraction of the total deposits belongs to the user.

        UserDepositShares= TotalBankDepositShares × Deposit Ratio


The formula used in your code for updating deposits with interest is:
       A = A₀ * eʳᵗ 
   
Interest growth follows an exponential function because the deposit balance is continuously compounded. This means that at every instant, the money earns interest not just on the initial deposit but also on the interest that has already been added.
where:

A = Final amount after interest is applied
A₀ =  Initial deposit amount
r = Interest rate (per unit time)
t = Time elapsed
𝑒 = Euler’s number( ≈2.718), which is the base of the natural logarithm

​ BORROW 
A user can borrow up to a borrowable limit, determined by the collateral value and liquidation threshold.
 The borrowed amount accrues interest over time.

A loan is considered safe as long  \as:

        Total Borrowed ≤ Collateral Value × Liquidation Threshold

Interest Accrual
Interest is compounded continuously using the formula:
        A = A₀ * eʳᵗ 




Liquidation is usually handled by liquidators, who are incentivized to repay a portion of the borrower's debt in exchange for discounted 
collateral.
When a liquidator repays a portion of the borrower's debt, they receive an equivalent amount of collateral + a liquidation bonus.
The liquidation bonus varies by asset (e.g., 5% for stablecoins, 10% for volatile assets).

Understanding Collateralization

Collateral: Assets a borrower deposits to secure a loan.

Borrowed Amount: The amount of assets a borrower has taken as a loan.

Health Factor: A ratio that determines if a position is at risk of liquidation.

      Health Factor = Total Collateral × Liquidation Threshold / Total Borrowed

 
Undercollateralization: If the Health Factor < 1, the borrower does not have enough collateral to support their loan, making them eligible for liquidation.

Overcollateralization: If Health Factor ≥ 1, the borrower has enough collateral and is safe from liquidation.

Liquidation Bonus: A discount on collateral that incentivizes liquidators to repay the borrower's debt.




1. Deposits & Withdrawals (Static Valuation)
When a user deposits collateral, they are just adding tokens to their balance.

When a user withdraws collateral, they are removing tokens within safe limits.

💡 Why no real-time price?

The protocol does not need to assess risk because:

Deposits don’t affect loan-to-value (LTV) immediately.

Withdrawals are only allowed if the remaining collateral is sufficient (already checked at deposit time).

2. Repayments (Debt Reduction)
When a user repays, they are reducing their borrowed amount.

Repaying lowers risk for both the borrower and protocol.

💡 Why no real-time price?

The protocol does not care about the market price when repaying—borrowed amounts are in fixed token units.

Example: If you borrowed 100 USDC, you must repay 100 USDC, regardless of USDC price fluctuations.

THINGS TO IMPROVE IN THE CODE :
this lending pool only handles sol and usdc, make it handle more tokens
here we have a fixed interest rate , make it dynamic 
multiple tokens as collateral 
For repaying havent considered the interest rate should make changes