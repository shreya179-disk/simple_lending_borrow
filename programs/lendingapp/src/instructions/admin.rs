use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::constants::*;
use crate::state::*;
// This is the admin of the entire lending protocol
#[derive(Accounts)]
pub struct InitializeBank<'info>{
    #[account(mut)]
    pub signer: Signer<'info>, 
    pub mint: InterfaceAccount<'info, Mint>,
    // on-chain account to store the Bank’s data
    #[account(                
        init,
        payer=signer,
        space= ANCHOR_DISCRIMINATOR_SIZE +DefiBank::INIT_SPACE,
        seeds= [mint.key().as_ref()],
        bump,    
    )]
    pub defibank: Account<'info, DefiBank> ,


    #[account(   // the tokens are stored in the token account controlled by the bank
        init, 
        token::mint = mint, 
        token::authority = bank_token_account,
        payer = signer,
        seeds = [b"treasury", mint.key().as_ref()],
        bump,
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>, 
    pub system_program: Program <'info, System>,
}
#[derive(Accounts)]
pub struct InitUser<'info>{
    #[account(mut)]
    pub signer: Signer<'info>, 
    #[account(   // the tokens are stored in the token account controlled by the bank
        init, 
        payer = signer,
        space= ANCHOR_DISCRIMINATOR_SIZE +User::INIT_SPACE,
        seeds = [signer.key().as_ref()],
        bump,
    )]
    pub user_acc:  Account<'info, User> ,
    pub system_program: Program <'info, System>,
}

pub fn process_init_bank(context:Context<InitializeBank>, liquidation_threshold:u64, max_ltv:u64) -> Result<()> {
    let bank = &mut context.accounts.defibank; // as we have already intialized we are calling it 
    bank.mint_address = context.accounts.mint.key();
    bank.authority= context.accounts.signer.key();
    bank.liquidation_threshold= liquidation_threshold;
    bank.max_ltv=max_ltv;
    Ok(())
}


pub fn process_user(context:Context<InitUser> , usdc_address:Pubkey) -> Result<()> {
    let user = & mut context.accounts.user_acc;
    user.owner = context.accounts.signer.key();
    user.usdc_address = usdc_address;
    
    Ok(())
}