use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Liquity Pool is locked!")]
    PoolLocked,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Slippage limits exceeded")]
    SlippageExceeded,
    #[msg("Curve Error")]
    CurveError,
    #[msg("Invalid authority")]
    InvalidAuthority,
}
