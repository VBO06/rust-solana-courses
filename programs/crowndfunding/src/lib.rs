// UDEMY Solana Blockchain Developer Bootcamp with Rust + JavaScript 

use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;


declare_id!("UAtcBu9QKREy592cS2EZPgSfmwaP1GCvD2Zo9dNPtYD");

#[program]
pub mod crowndfunding {
    use super::*;

    pub fn create(ctx: Context<Create>, name: String, description: String) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
        campaign.name = name;
        campaign.description = description;
        campaign.amount_donated = 0;
        campaign.admin = *ctx.accounts.user.key;
        Ok(())
    }
    
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
        let user = &mut ctx.accounts.user;

        if campaign.admin != *user.key {
            return Err(ProgramError::IncorrectProgramId);
        }
        // Minimum required for campaign (like blocked funds)
        let rent_balance = Rent::get()?.minimum_balance(campaign.to_account_info().data_len());

        // Money of campaign - rent balance is ok and amount must be < 
        if **campaign.to_account_info().lamports.borrow() - rent_balance < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        // à voir comment update amount_donated (&mut ctx.accounts.campaign).amount_donated -= amount;
        // Withdraw from campaign 
        **campaign.to_account_info().try_borrow_mut_lamports()? -= amount;
        // Add to user
        **user.to_account_info().try_borrow_mut_lamports()? += amount;
        // MAJ amount donate when withdraw
        campaign.amount_donated -= amount;
        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> ProgramResult {
       let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.campaign.key(),
        amount
       );
       anchor_lang::solana_program::program::invoke(
        &ix, 
        &[
                            ctx.accounts.user.to_account_info(),
                            ctx.accounts.campaign.to_account_info(),
                        ]
        );
        (&mut ctx.accounts.campaign).amount_donated += amount;
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer=user, space=9000, seeds=["CAMPAIGN_DEMO".as_ref(), user.key().as_ref()], bump)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct Campaign { 
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
    pub amount_donated: u64
}