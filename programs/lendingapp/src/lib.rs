pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

declare_id!("CF6a2Y8jpYC8Jc2a6bXTro4dQ462N6HJ7hQxGsaLr7Pa");

#[program]
pub mod lendingapp {

    use super::*;

    pub fn initbank(ctx: Context<InitializeBank>, liquidation_threshold:u64, max_ltv:u64) -> Result<()> {
       process_init_bank(ctx, liquidation_threshold, max_ltv)
    }

    pub fn inituser(ctx: Context<InitUser> ,usdc_address:Pubkey) -> Result<()> {
        process_user(ctx, usdc_address)
    } 

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        process_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        process_withdraw(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        process_borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        process_repay(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        process_liquidation(ctx)
    }
}