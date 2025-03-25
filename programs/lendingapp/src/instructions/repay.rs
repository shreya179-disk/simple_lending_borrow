use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};
use crate::state::*;
use std::f32::consts::E;
use crate::error::ErrorCode;
//basically user repays the borrowed token back to the bank almost same format as deposit 
#[derive(Accounts)]
pub struct Repay<'info>{
  #[account(mut)]
  pub signer: Signer<'info>, 
  pub mint: InterfaceAccount<'info, Mint>,
  

    #[account(            
        mut, 
        seeds = [signer.key().as_ref()],
        bump,
    )]  
    pub user_acc:  Account<'info, User> ,

    #[account(                             
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,        //token address where token are presemt to put it the bank 
        associated_token::authority = signer,
        associated_token::token_program = token_program//This ensures that the ATA was created using the correct SPL Token Program.
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    
    #[account(
        mut, 
        seeds = [mint.key().as_ref()],
        bump,
    )]  
    pub defibank: Account<'info, DefiBank>,
    #[account(
        mut, 
        seeds = [b"treasury", mint.key().as_ref()],
        bump, 
    )]  
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,


}

 pub fn process_repay(ctx: Context<Repay> , amount:u64) -> Result<()> {
    let bank = &mut ctx.accounts.defibank;
    let user =&mut ctx.accounts.user_acc;
    let borrowed_asset_value:u64; 
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            borrowed_asset_value = user.borrowed_usdc;
        },
        _ => {
            borrowed_asset_value = user.borrowed_sol;
        }
    }
    let time_diff = Clock::get()?.unix_timestamp - user.last_update_borrowed ;
    bank.total_borrowed = ((bank.total_borrowed as f64) * (E as f64).powf(bank.interest_rate as f64 * time_diff as f64)) as u64;
    let value_per_share = bank.total_borrowed as f64 /bank.total_borrowed_shares as f64;
    let user_value = borrowed_asset_value as f64/value_per_share;
    if user_value < amount as f64 {
        return Err(ErrorCode::OverRepay.into());
    }
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;
    


    let borrow_ratio = amount.checked_div(bank.total_borrowed).unwrap(); //same logic as deposit 
    let users_shares = bank.total_borrowed_shares.checked_mul(borrow_ratio).unwrap();
      
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            user.borrowed_usdc -= amount;
            user.deposited_usdc_shares -= users_shares;
        },
        _ => {
            user.borrowed_sol -= amount;
            user.deposited_sol_shares -= users_shares;
        }
    }
    bank.total_borrowed -= amount;
    bank.total_borrowed_shares -= users_shares;
    
    Ok(())
}