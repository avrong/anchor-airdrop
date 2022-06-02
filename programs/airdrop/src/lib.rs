use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer, TokenAccount};
use anchor_spl::associated_token::{Create, create, get_associated_token_address};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod airdrop {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64, token_mint: Pubkey) -> Result<()> {
        // Simple transfer
        // let cpi_accounts = Transfer {
        //     from: ctx.accounts.from.to_account_info().clone(),
        //     to: ctx.accounts.to.to_account_info().clone(),
        //     authority: ctx.accounts.initializer.clone(),
        // };
        // let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info().clone(), cpi_accounts);
        // token::transfer(cpi_context, amount)?;

        // Simple ATA transfer
        let calculated_ata_account = get_associated_token_address(
            ctx.accounts.to_main.to_account_info().key,
            &token_mint,
        );

        assert_eq!(&calculated_ata_account, ctx.accounts.to_token.to_account_info().key);

        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.to_token.to_account_info().clone(),
            authority: ctx.accounts.initializer.to_account_info().clone(),
        };
        let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info().clone(), cpi_accounts);
        token::transfer(cpi_context, amount)?;

        // Transfer to remaining_accounts
        // let provided_remaining_accounts = &mut ctx.remaining_accounts.chunks(2);
        // for pair in provided_remaining_accounts {
        //     let pair_iter = &mut pair.iter();
        //     let main_account_info = next_account_info(pair_iter)?;
        //     let ata_account_info = next_account_info(pair_iter)?;

        //     // TODO: check there are no same addressess
        //     // TODO: check account is writable
        //     // TODO: check owner is SystemProgram
        //     // TODO: check owner is TokenProgram
        //     // TODO: autority & mint are equal

        //     // Checking if there is associated token account & create if not
        //     let calculated_ata_account = get_associated_token_address(
        //         main_account_info.key,
        //         &token_mint,
        //     );

        //     let cpi_accounts = Transfer {
        //         from: ctx.accounts.from.to_account_info().clone(),
        //         to: ata_account_info.clone(),
        //         authority: ctx.accounts.initializer.to_account_info().clone(),
        //     };
        //     let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info().clone(), cpi_accounts);
        //     token::transfer(cpi_context, amount)?;
        // }

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
    #[account(mut)]
    pub to_main: AccountInfo<'info>,

    #[account(mut)]
    pub to_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Airdrop {}
