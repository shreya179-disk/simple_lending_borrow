use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]

pub struct User{
    pub owner: Pubkey,
    pub deposited_sol: u64,
    pub deposited_sol_shares: u64,
    pub borrowed_sol: u64,
    pub borrowed_sol_shares: u64, 
    pub deposited_usdc: u64,
    pub deposited_usdc_shares: u64, 
    pub borrowed_usdc: u64,
    pub borrowed_usdc_shares: u64,
    pub usdc_address: Pubkey,//just 2 tokens so easy to apply if statement
    pub last_updated: i64,
}
//each asset is gonna have its own bank 
#[account]
#[derive(InitSpace)]
pub struct DefiBank{
    pub authority: Pubkey, //who will have authority to change the configs of the bank (special permissions)
    pub mint_address: Pubkey,
    pub total_deposits: u64,
    pub total_deposit_shares: u64,
    pub total_borrowed: u64,
    pub  total_borrowed_shares: u64,
    //e LTV (Loan-to-Value) ratio beyond which an account can be liquidated.
    pub liquidation_threshold: u64,
    ///  Extra collateral the liquidator can claim.
    pub liquidation_bonus: u64,
    /// Percentage of collateral that can be liquidated
    pub liquidation_close_factor: u64,
    /// Max percentage of collateral that can be borrowed
    pub max_ltv: u64,
    pub last_updated: i64,
   
}