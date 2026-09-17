use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,

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

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);
        require!(amount > 0, AmmError::InvalidAmount);

        // 1. Calculate protocol fee amount
        let protocol_fee_amount = (amount as u128)
            .checked_mul(self.config.protocol_fee as u128)
            .ok_or(AmmError::CurveError)?
            .checked_div(10_000)
            .ok_or(AmmError::CurveError)? as u64;

        // 2. Amount that goes into the curve (after total fee)
        let amount_after_fee = (amount as u128)
            .checked_mul((10_000 - self.config.fee as u128))
            .ok_or(AmmError::CurveError)?
            .checked_div(10_000)
            .ok_or(AmmError::CurveError)? as u64;



        // Calculate output=
        let amount_out = if is_x {
            // Selling X → receive Y
            ConstantProduct::delta_y_from_x_swap_amount(
                self.vault_x.amount,
                self.vault_y.amount,
                amount_after_fee,
            )
            .map_err(|_| AmmError::CurveError)?
        } else {
            // Selling Y → receive X
            ConstantProduct::delta_x_from_y_swap_amount(
                self.vault_x.amount,
                self.vault_y.amount,
                amount_after_fee,
            )
            .map_err(|_| AmmError::CurveError)?
        };

        require!(amount_out >= min, AmmError::SlippageExceeded);

        if is_x {
            // User sends X → vault, receives Y from vault
            self.transfer_user_to_vault(true, amount)?;                    // full amount into vault
            self.transfer_vault_to_treasury(true, protocol_fee_amount)?;   // send protocol fee to treasury
            self.transfer_vault_to_user(false, amount_out)?;

        } else {
            // User sends Y → vault, receives X from vault
            self.transfer_user_to_vault(false, amount)?;
            self.transfer_vault_to_treasury(false, protocol_fee_amount)?;
            self.transfer_vault_to_user(true, amount_out)?;
        }

        Ok(())
    }

    fn transfer_vault_to_treasury(&self, is_x: bool, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }

        let (from, to) = if is_x {
            (
                self.vault_x.to_account_info(),
                self.treasury_x.to_account_info(),
            )
        } else {
            (
                self.vault_y.to_account_info(),
                self.treasury_y.to_account_info(),
            )
        };

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

    fn transfer_user_to_vault(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = if is_x {
            (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
            )
        } else {
            (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
            )
        };

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.token_program.key(),
            cpi_accounts,
        );

        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    fn transfer_vault_to_user(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = if is_x {
            (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
            )
        } else {
            (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            )
        };

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

pub fn handle_swap(ctx: Context<Swap>, is_x: bool, amount: u64, min: u64) -> Result<()> {
    ctx.accounts.swap(is_x, amount, min)
}