use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer, TokenAccount, Mint};
use anchor_spl::associated_token::{self, AssociatedToken, Create};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod airdrop {
    use super::*;

    pub fn transfer_simple(ctx: Context<TransferSimple>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.initializer.to_account_info().clone(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            cpi_accounts
        );
        token::transfer(cpi_context, amount)?;

        Ok(())
    }

    pub fn transfer_ata(ctx: Context<TransferATA>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.to_ata.to_account_info().clone(),
            authority: ctx.accounts.initializer.to_account_info().clone(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            cpi_accounts
        );
        token::transfer(cpi_context, amount)?;

        Ok(())
    }

    pub fn airdrop<'a>(ctx: Context<'_, '_, '_, 'a, Airdrop<'a>>, amount: u64) -> Result<()> {
        let remaining_accounts = ctx.remaining_accounts.chunks(2);

        for pair in remaining_accounts {
            let (main_acc, ata) = (&pair[0], &pair[1]);

            let calculated_ata = associated_token::get_associated_token_address(
                main_acc.to_account_info().key,
                ctx.accounts.mint.to_account_info().key,
            );

            if ata.data_is_empty() {
                let cpi_context = ctx.accounts.into_create_ata_context(
                    ata.clone(),
                    main_acc.clone(),
                );
                associated_token::create(cpi_context)?;
            }

            assert_eq!(ata.key, &calculated_ata);
            assert!(!ata.data_is_empty());

            let cpi_context = ctx.accounts.into_transfer_context(ata.clone());
            token::transfer(cpi_context, amount)?;
        }

        // TODO: check there are no same addressess
        // TODO: check account is writable
        // TODO: check owner is SystemProgram
        // TODO: check owner is TokenProgram
        // TODO: autority & mint are equal

        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferSimple<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut, constraint = from.amount > to.amount)]
    pub from: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferATA<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(mut, token::mint = mint, token::authority = initializer)]
    pub from: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to_main: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint,
        associated_token::authority = to_main
    )]
    pub to_ata: Account<'info, TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Airdrop<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(mut)]
    pub from: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Airdrop<'info> {
    fn into_create_ata_context(
        &self,
        ata: AccountInfo<'info>,
        main_acc: AccountInfo<'info>
    ) -> CpiContext<'_, '_, '_, 'info, Create<'info>> {
        let cpi_accounts = Create {
            payer: self.initializer.to_account_info().clone(),
            associated_token: ata.to_account_info().clone(),
            authority: main_acc.to_account_info().clone(),
            mint: self.mint.to_account_info().clone(),
            system_program: self.system_program.to_account_info().clone(),
            token_program: self.token_program.to_account_info().clone(),
            rent: self.rent.to_account_info().clone(),
        };

        CpiContext::new(
            self.associated_token_program.to_account_info().clone(),
            cpi_accounts
        )
    }

    fn into_transfer_context(
        &self,
        to: AccountInfo<'info>
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info().clone(),
            to: to.to_account_info().clone(),
            authority: self.initializer.to_account_info().clone(),
        };

        CpiContext::new(
            self.token_program.to_account_info().clone(),
            cpi_accounts
        )
    }
}
