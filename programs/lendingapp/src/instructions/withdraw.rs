

use std::f32::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};
use crate::state::*;
use crate::error::ErrorCode;

// now the user wants to withdraw  things it requires , so needs to pay apy on these funds
//needs to verify if the user does have balance of that particular token 
// first intialize the bank and from the token account if the bank we are tranferring to the user token account 
//state of the bank and user account also changes 
// also the total shares

#[derive(Accounts)]
pub struct Withdraw<'info>{
    #[account(mut)]
    pub signer: Signer<'info>, 
    pub mint: InterfaceAccount<'info, Mint>,

    // now we need the bank which has token account to which the user is trying to withdraw from
    #[account(
    mut,
    seeds= [mint.key().as_ref()],
    bump)]  
    pub defibank: Account<'info, DefiBank> ,
    // The bank's token account from which tokens will be withdrawn
    #[account(
    mut,
    seeds= [b"treasury", mint.key().as_ref()],
    bump)] 
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,
    //state of the user account 
    #[account(            
        mut, 
        seeds = [signer.key().as_ref()],
        bump,
    )]  
    pub user_acc:  Account<'info, User> ,
    //The user's token account where the withdrawn tokens will be transferred
    #[account(                             
        mut, 
        associated_token::mint = mint,        //token address where token are presemt to put it the bank 
        associated_token::authority = signer,
        associated_token::token_program = token_program//This ensures that the ATA was created using the correct SPL Token Program.
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>

}


pub fn process_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let bank = &mut ctx.accounts.defibank;
    let user = &mut ctx.accounts.user_acc;
    let deposited_value: u64;
    if ctx.accounts.mint.to_account_info().key() == user.usdc_address {
        deposited_value = user.deposited_usdc;
    } else {
        deposited_value = user.deposited_sol;
    }


    // Check if the user has sufficient funds based on the mint typ
    
    let time_diff = Clock::get()?.unix_timestamp - user.last_updated ;
    let updated_deposits = (bank.total_deposits as f64) * (E as f64).powf(bank.interest_rate as f64 * time_diff as f64);
    bank.total_deposits = updated_deposits as u64;
    let value_per_share = bank.total_deposits as f64 /bank.total_deposit_shares as f64;
    let user_value = deposited_value as f64/value_per_share;
    if user_value < amount as f64 {
        return Err(ErrorCode::InsufficientFunds.into());
    }
    // Calculate the shares to remove based on the withdrawal amount
    let shares_to_remove = (amount as f64 / bank.total_deposits as f64) * bank.total_deposit_shares as f64;
    let shares_to_remove = shares_to_remove as u64; // Convert to u64 for further calculations

    // Update the user's deposited balance and shares based on the mint type
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            user.deposited_usdc -= amount;
            user.deposited_usdc_shares -= shares_to_remove;
        }
        _ => {
            user.deposited_sol -= amount;
            user.deposited_sol_shares -= shares_to_remove;
        }
    }

    // Update the bank's total deposits and shares
    bank.total_deposits -= amount;
    bank.total_deposit_shares -= shares_to_remove;

    // Prepare the CPI (Cross-Program Invocation) accounts for the token transfer
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.bank_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.bank_token_account.to_account_info(),
    };

    // Get the token program account info
    let cpi_program = ctx.accounts.token_program.to_account_info();

    // Prepare the signer seeds for the CPI context
    // But when a user withdraws funds, the PDA must authorize the transfer out (like opening the mailbox to take letters out). Since PDAs don’t have private keys, they use signer seeds to prove ownership
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury",
            ctx.accounts.mint.to_account_info().key.as_ref(),
            &[ctx.bumps.bank_token_account],
        ],
    ];

    // Create the CPI context with the signer seeds
    let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);

    // Get the decimals of the mint
    let decimals = ctx.accounts.mint.decimals;

    // Perform the token transfer using the CPI
    token_interface::transfer_checked(cpi_context, amount, decimals)?;

    // Update the last updated timestamp
    user.last_updated = Clock::get()?.unix_timestamp;

    Ok(())
}