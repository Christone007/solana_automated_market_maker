pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

// pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5Sp6wXLRaexwd5UCzh3eW2VRTNQuCPRaH7K4zGzecKq4");

#[program]
pub mod amm2026 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed:u64, fee: u16, protocol_fee: u16, authority:Option<Pubkey>) -> Result<()> {
        handle_initialize(ctx, seed, fee, protocol_fee, authority)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x:u64, max_y: u64) -> Result<()> {
        handle_deposit(ctx, amount, max_x, max_y)
    }
}
