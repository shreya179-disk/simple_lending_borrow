use std::f32::consts::E;
use crate::{calculate_accrued_interest, error::ErrorCode};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use crate:: constants::*;
use crate::state::*;
//now user uses the deposited token as a colloateral to borrow tokens
// should be overcollateralized 
//calculate collateral so that  they can borrow against 
// bank account and token account of the borrowing token
#[derive(Accounts)]
pub struct Borrow<'info>{
  #[account(mut)]
  pub signer: Signer<'info>, 
  pub mint: InterfaceAccount<'info, Mint>,
   // bank account of the token user wants to borrow 
  #[account(
    mut,
    seeds= [mint.key().as_ref()],
    bump)]  
    pub defibank: Account<'info, DefiBank> ,
    // now you also need the token account of the bank the user wants to borrow
    #[account(
    mut,
    seeds= [b"treasury", mint.key().as_ref()],
    bump)] 
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(              // the state of the account of user is also gonna change
        mut, 
        seeds = [signer.key().as_ref()],
        bump,
    )]  
    pub user_acc:  Account<'info, User> ,

    #[account( 
        init_if_needed,  //a user might not have borrowed this token before, meaning their associated token account may not exist yet.
        payer = signer,
        associated_token::mint = mint, 
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>, 
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program <'info, System>,

    pub price_update: Account<'info, PriceUpdateV2>,

} 


pub fn process_borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    // Check if user has enough collateral to borrow
    let bank = &mut ctx.accounts.defibank;
    let user = &mut ctx.accounts.user_acc;

    let price_update = &mut ctx.accounts.price_update;

    let total_collateral: u64;

    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => { //if the user is borrowing usdc, we need to calculate the collateral 
            let sol_feed_id = get_feed_id_from_hex(FEED_ID_SOL_USD)?; 
            let sol_price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &sol_feed_id)?;
            let accrued_interest = calculate_accrued_interest(user.deposited_sol, bank.interest_rate, user.last_updated)?;
            total_collateral = sol_price.price as u64 * (user.deposited_sol + accrued_interest); //
        },
        _ => {
            let usdc_feed_id = get_feed_id_from_hex(FEED_ID_USDC_USD)?;
            let usdc_price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &usdc_feed_id)?;
            total_collateral = usdc_price.price as u64 * user.deposited_usdc;

        }
    }

    let borrowable_amount = total_collateral as u64 *  bank.liquidation_threshold;

    if borrowable_amount < amount {
        return Err(ErrorCode::OverBorrowableAmount.into());
    }       

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.bank_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.bank_token_account.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let mint_key = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury",
            mint_key.as_ref(),
            &[ctx.bumps.bank_token_account],
        ],
    ];
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);
    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    if bank.total_borrowed == 0 {
        bank.total_borrowed = amount;
        bank.total_borrowed_shares = amount;
    } 

    let borrow_ratio = amount.checked_div(bank.total_borrowed).unwrap(); //same logic as deposit 
    let users_shares = bank.total_borrowed_shares.checked_mul(borrow_ratio).unwrap();

    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            user.borrowed_usdc += amount;
            user.deposited_usdc_shares += users_shares;
        },
        _ => {
            user.borrowed_sol += amount;
            user.deposited_sol_shares += users_shares;
        }
    }
    bank.total_borrowed += amount;
    bank.total_borrowed_shares += users_shares;
    
    user.last_update_borrowed = Clock::get()?.unix_timestamp;
    Ok(())
}
