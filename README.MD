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

A loan is considered safe as long as:

Total Borrowed ≤ Collateral Value × Liquidation Threshold

Interest Accrual
Interest is compounded continuously using the formula:
       A = A₀ * eʳᵗ 




Liquidation is usually handled by liquidators, who are incentivized to repay a portion of the borrower's debt in exchange for discounted 
collateral.
When a liquidator repays a portion of the borrower's debt, they receive an equivalent amount of collateral + a liquidation bonus.
The liquidation bonus varies by asset (e.g., 5% for stablecoins, 10% for volatile assets).


THINGS TO IMPROVE IN THE CODE :
this lending pool only handles sol and usdc, make it handle more tokens
here we have a fixed interest rate , make it dynamic 
multiple tokens as collateral 
For repaying havent considered the interest rate should make changes