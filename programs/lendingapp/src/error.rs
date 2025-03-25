use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for withdrawal.")]
    InsufficientFunds,

    #[msg("Amount exceeds borrowable limit.")]
    OverBorrowableAmount,
}
