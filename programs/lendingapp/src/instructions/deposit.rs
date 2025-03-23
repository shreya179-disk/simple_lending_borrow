use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};

use crate::state::*;

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(mut)]
    pub signer: Signer<'info>, 
    pub mint: InterfaceAccount<'info, Mint>,

    // now we need the bank which has token account to which the user is trying to deposit the token into 
    #[account(
    mut,
    seeds= [mint.key().as_ref()],
    bump)]  
    pub defibank: Account<'info, DefiBank> ,
    // now you have deposited the token into the bank you also need the token account of the bank
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
    //ensures that the user has the token
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

//cpi transfer from token account of the user to the bank 
// new shares added to the bank 
//new shares added to the user 
// update users deposited and collateral value 
// update the state of bank as well 

pub fn process_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
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

    // Update the state of both the user and bank account
    let bank = &mut ctx.accounts.defibank;

    // Use checked operations for underflow and overflow
    if bank.total_deposits == 0 {
        bank.total_deposits = amount;
        bank.total_deposit_shares = amount;
    }

    // Calculate the deposit ratio
    let deposit_ratio = amount.checked_div(bank.total_deposit_shares).unwrap();

    // Calculate the user's shares
    let users_shares = bank.total_deposit_shares.checked_mul(deposit_ratio).unwrap_or(0); // Provide a fallback value

    let user = &mut ctx.accounts.user_acc;

    //  If you only need the public key → Use ctx.accounts.mint.key().
    // If you need full account info (like ownership, lamports, etc.) → Use ctx.accounts.mint.to_account_info().key().
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            user.deposited_usdc += amount;
            user.deposited_usdc_shares += users_shares; 
        },
        _ => {
            user.deposited_sol += amount;
            user.deposited_sol_shares += users_shares; 
        }
    }

    bank.total_deposits += amount;
    bank.total_deposit_shares += users_shares; // Now users_shares is a u64, not an Option<u64>

    // Update the last updated timestamp
    user.last_updated = Clock::get()?.unix_timestamp;

    Ok(())

}