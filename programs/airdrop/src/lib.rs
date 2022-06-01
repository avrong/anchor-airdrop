use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod airdrop {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.initializer.clone(),
        };
        let cpi_context = CpiContext::new(ctx.accounts.token_program.clone(), cpi_accounts);
        token::transfer(cpi_context, 42);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK: ???
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,

    #[account(mut, constraint = from.amount > to.amount)]
    pub from: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// CHECK: ???
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Airdrop {}
