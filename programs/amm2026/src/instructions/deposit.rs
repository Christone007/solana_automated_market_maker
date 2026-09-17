use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo,Token, TokenAccount, Transfer}
};
use constant_product_curve::{ConstantProduct, CurveError, XYAmounts};
use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user
    )]
    pub user_lp: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

pub fn deposit_tokens(ctx: &Context<Deposit>, is_x: bool, amount: u64) -> Result<()> {
    let (from, to) = match is_x {
        true => (
            ctx.accounts.user_x.to_account_info(),
            ctx.accounts.vault_x.to_account_info()
        ),
        false => (
            ctx.accounts.user_y.to_account_info(),
            ctx.accounts.vault_y.to_account_info()
        )
    };

    let cpi_program = ctx.accounts.token_program.key();

    let cpi_accounts = Transfer {
        from: from,
        to: to,
        authority: ctx.accounts.user.to_account_info()
    };

    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

    let _ = transfer(cpi_context, amount);

    Ok(())
}

pub fn mint_lp_tokens (ctx: &Context<Deposit>, amount: u64) -> Result<()> {
    let cpi_program = ctx.accounts.token_program.key();
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint_lp.to_account_info(),
        to: ctx.accounts.user_lp.to_account_info(),
        authority: ctx.accounts.config.to_account_info()
    };

    let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &ctx.accounts.config.seed.to_le_bytes(),
            &[ctx.accounts.config.config_bump]
        ]];


    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    
    let _ = mint_to(cpi_ctx, amount);

    Ok(())
}

pub fn handle_deposit(ctx:Context<Deposit>, amount:u64, max_x: u64, max_y: u64) -> Result<()> {
    require!(!ctx.accounts.config.locked, AmmError::PoolLocked);
    require_neq!(amount, 0, AmmError::InvalidAmount);

    let (x, y) = if ctx.accounts.mint_lp.supply == 0 && ctx.accounts.vault_x.amount == 0 && ctx.accounts.vault_y.amount == 0 {
        (max_x, max_y)
    } else {
        let amounts = ConstantProduct::xy_deposit_amounts_from_l(
            ctx.accounts.vault_x.amount,
            ctx.accounts.vault_y.amount,
            ctx.accounts.mint_lp.supply,
            amount,
            6).unwrap();

        require!(
            amounts.x <= max_x && amounts.y <= max_y,
            AmmError::SlippageExceeded
        );

        (amounts.x, amounts.y)
    };

    // deposit x
    let _ = deposit_tokens(&ctx,true, x);                                                                                      

    // deposit y
    let _ = deposit_tokens(&ctx, false, y);    

    // claim lp tokens
    let _ = mint_lp_tokens(&ctx, amount);

    Ok(())
}
