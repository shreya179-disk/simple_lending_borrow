pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

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
}
