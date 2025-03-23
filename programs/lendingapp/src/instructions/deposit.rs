use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::TransferChecked, token_interface::{self, Mint, TokenAccount, TokenInterface}};

use crate::{DefiBank, User};

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
    // now you have deposited the token into the bank you also need the token account 
    #[account(
    mut,
    seeds= [b"treasury", mint.key().as_ref()],
    bump)] 
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(    //state of the user account
        mut, 
        seeds = [signer.key().as_ref()],
        bump,
    )]  
    pub user_acc:  Account<'info, User> ,

    #[account(                             //ensures that the user has the token
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
    Ok(())
}
