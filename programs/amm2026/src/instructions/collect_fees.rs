use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = config.authority == Some(authority.key()) @ AmmError::InvalidAuthority
    )]
    pub config: Account<'info, Config>,

    // Treasury accounts (owned by config PDA)
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub treasury_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub treasury_y: Box<Account<'info, TokenAccount>>,

    // Destination ATAs owned by the authority
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = authority
    )]
    pub authority_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = authority
    )]
    pub authority_y: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> CollectFees<'info> {
    pub fn collect(&self, amount_x: u64, amount_y: u64) -> Result<()> {
        // Withdraw token X from treasury
        if amount_x > 0 {
            self.transfer_from_treasury(
                self.treasury_x.to_account_info(),
                self.authority_x.to_account_info(),
                amount_x,
            )?;
        }

        // Withdraw token Y from treasury
        if amount_y > 0 {
            self.transfer_from_treasury(
                self.treasury_y.to_account_info(),
                self.authority_y.to_account_info(),
                amount_y,
            )?;
        }

        Ok(())
    }

    fn transfer_from_treasury(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };

        let seeds: &[&[u8]] = &[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.key(),
            cpi_accounts,
            signer_seeds,
        );

        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

pub fn handle_collect_fees(
    ctx: Context<CollectFees>,
    amount_x: u64,
    amount_y: u64,
) -> Result<()> {
    ctx.accounts.collect(amount_x, amount_y)
}