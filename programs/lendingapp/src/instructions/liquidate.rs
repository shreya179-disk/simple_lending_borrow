use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};
use crate::{state::*, FEED_ID_SOL_USD, FEED_ID_USDC_USD, MAXIMUM_AGE};
use crate::error::ErrorCode;
use crate::utils::calculate_accrued_interest;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

//Liquidation is usually handled by liquidators, who are incentivized to repay a portion of the borrower's debt in exchange for discounted collateral.
#[derive(Accounts)]
pub struct Liquidate<'info>{
  #[account(mut)]
   pub liquidator: Signer<'info>, 
   pub price_update: Account<'info, PriceUpdateV2>,
   pub collateral_mint: InterfaceAccount<'info, Mint>,//During liquidation, this is the token that the liquidator will receive at a discounted price
   pub borrowed_mint: InterfaceAccount<'info, Mint>,//The liquidator needs to repay a portion of the borrowed amount to trigger the liquidation

   #[account(
        mut, 
        seeds = [collateral_mint.key().as_ref()],
        bump,
    )]  
    pub collateral_bank: Account<'info, DefiBank>,//The collateral bank is the bank that the liquidator will receive the collateral from
    #[account(
        mut, 
        seeds = [b"treasury", collateral_mint.key().as_ref()],
        bump, 
    )]  
    pub collateral_bank_token_account: InterfaceAccount<'info, TokenAccount>,//The collateral bank token account is the token account that the liquidator will receive the collateral from
    #[account(
        mut, 
        seeds = [borrowed_mint.key().as_ref()],
        bump,
    )]  
    pub borrowed_bank: Account<'info, DefiBank>,
    #[account(
        mut, 
        seeds = [b"treasury", borrowed_mint.key().as_ref()],
        bump, 
    )]  
    pub borrowed_bank_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut, 
        seeds = [liquidator.key().as_ref()],
        bump,
    )]  
    pub user_account: Account<'info, User>,
    #[account( 
        init_if_needed, 
        payer = liquidator,
        associated_token::mint = collateral_mint, 
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_collateral_token_account: InterfaceAccount<'info, TokenAccount>, 
    #[account( 
        init_if_needed, 
        payer = liquidator,
        associated_token::mint = borrowed_mint, 
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    pub liquidator_borrowed_token_account: InterfaceAccount<'info, TokenAccount>, 
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

}

pub fn process_liquidation(ctx: Context<Liquidate>) -> Result<()> {
    let collateral_bank = &ctx.accounts.collateral_bank;
    let borrowed_bank = &ctx.accounts.borrowed_bank;
    let user_account = &ctx.accounts.user_account; //the liquidator account intialized
    let price_update = &mut ctx.accounts.price_update;
    
    let sol_feed_id = get_feed_id_from_hex(FEED_ID_SOL_USD)?;
    let usdc_feed_id = get_feed_id_from_hex(FEED_ID_USDC_USD)?;
    let sol_price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &sol_feed_id)?;
    let usdc_price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &usdc_feed_id)?;

    let total_collateral: u64;
    let total_borrowed: u64;

    match ctx.accounts.collateral_mint.to_account_info().key() {  
        key if key == user_account.usdc_address => {  // now the user  has deposited usdc as collateral 
           let new_usdc  = calculate_accrued_interest(user_account.deposited_usdc, collateral_bank.interest_rate, user_account.last_updated)?;
           total_collateral = usdc_price.price as u64 * new_usdc;
           let new_sol = calculate_accrued_interest(user_account.borrowed_sol, borrowed_bank.interest_rate, user_account.last_update_borrowed)?;
           total_borrowed = sol_price.price as u64 * new_sol;
    
        }
        _ => {  //if the  user has deposited sol as collateral
            let new_sol = calculate_accrued_interest(user_account.deposited_sol, collateral_bank.interest_rate, user_account.last_updated)?;
            total_collateral = sol_price.price as u64 * new_sol;
            let new_usdc = calculate_accrued_interest(user_account.borrowed_usdc, borrowed_bank.interest_rate, user_account.last_update_borrowed)?;
            total_borrowed = usdc_price.price as u64 * new_usdc;
        }
     }
     let health_factor = (total_collateral * collateral_bank.liquidation_threshold).checked_div(total_borrowed).unwrap();
     if health_factor >= 1 {
       return Err(ErrorCode::NotUnderCollateralized.into());
     }
     //Transfer #1: The Liquidator Repays a Portion of the Borrower's Debt

     let transfer_to_bank = TransferChecked{
        from: ctx.accounts.liquidator_borrowed_token_account.to_account_info(),
        to: ctx.accounts.borrowed_bank_token_account.to_account_info(),
        mint: ctx.accounts.borrowed_mint.to_account_info(),
        authority: ctx.accounts.liquidator.to_account_info(),
    };
    //The liquidator repays a portion of the borrower’s debt.The amount repaid is determined by the liquidation close factor 
    let liquidation_amount = total_borrowed.checked_mul(borrowed_bank.liquidation_close_factor).unwrap();

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_to_bank);
    token_interface::transfer_checked(cpi_ctx, liquidation_amount, ctx.accounts.borrowed_mint.decimals)?;

    //Transfer #2: The Liquidator Receives Collateral in Return
    let liquidator_amount  =(liquidation_amount* collateral_bank.liquidation_bonus) + liquidation_amount;
    let transfer_to_liquidator = TransferChecked{
        from: ctx.accounts.collateral_bank_token_account.to_account_info(),
        to: ctx.accounts.liquidator_collateral_token_account.to_account_info(),
        mint: ctx.accounts.collateral_mint.to_account_info(),
        authority: ctx.accounts.collateral_bank_token_account.to_account_info(),
    };
    let mint_key = ctx.accounts.collateral_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury",
            mint_key.as_ref(),
            &[ctx.bumps.collateral_bank_token_account],
        ],
    ];
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_to_liquidator).with_signer(signer_seeds);
    token_interface::transfer_checked(cpi_ctx, liquidator_amount, ctx.accounts.collateral_mint.decimals)?;

    Ok(())
}  