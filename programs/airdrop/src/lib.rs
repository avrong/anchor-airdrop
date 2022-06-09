use std::collections::HashSet;
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
            cpi_accounts,
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
            cpi_accounts,
        );
        token::transfer(cpi_context, amount)?;

        Ok(())
    }

    pub fn airdrop<'a>(ctx: Context<'_, '_, '_, 'a, Airdrop<'a>>, amount: u64) -> Result<()> {
        // TODO: Maybe filter duplicates here once?
        let mut processed_accounts = HashSet::new();

        let remaining_accounts = ctx.remaining_accounts.chunks(2);
        for pair in remaining_accounts {
            let (main_acc, ata) = (&pair[0], &pair[1]);

            let calculated_ata = associated_token::get_associated_token_address(
                main_acc.to_account_info().key,
                ctx.accounts.mint.to_account_info().key,
            );

            // TODO: Should we check that account is writable after creation try?
            // When we're creating it, it is always writable.
            if !ata.is_writable {
                return Err(Error::from(ErrorCode::AccountNotMutable));
            }

            // Check key is the same as calculated
            if ata.key != &calculated_ata {
                return Err(Error::from(ErrorCode::AccountNotAssociatedTokenAccount));
            }

            if ata.data_is_empty() {
                let cpi_context = ctx.accounts.into_create_ata_context(
                    ata.clone(),
                    main_acc.clone(),
                );
                associated_token::create(cpi_context)?;
            }

            // Just another check for the fact that account is created.
            if ata.data_is_empty() {
                return Err(Error::from(ErrorCode::AccountNotInitialized));
            }

            // Check that accounts' owners are right
            if main_acc.owner != &System::id() {
                return Err(Error::from(ErrorCode::AccountNotSystemOwned));
            }
            if ata.owner != &Token::id() {
                return Err(Error::from(ErrorCode::AccountNotAssociatedTokenAccount));
            }

            // Check that mint is right & owner is main account
            let pa: Account<TokenAccount> = Account::try_from_unchecked(&ata)?;
            if pa.mint != ctx.accounts.mint.to_account_info().key() {
                return Err(Error::from(ErrorCode::ConstraintTokenMint));
            }
            if &pa.owner != main_acc.key {
                return Err(Error::from(ErrorCode::ConstraintTokenOwner));
            }

            // Skip duplicates
            if processed_accounts.contains(ata.key) {
                continue
            }

            let cpi_context = ctx.accounts.into_transfer_context(ata.clone());
            token::transfer(cpi_context, amount)?;

            processed_accounts.insert(ata.key);
        }

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
