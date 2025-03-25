use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for withdrawal.")]
    InsufficientFunds,

    #[msg("Amount exceeds borrowable limit.")]
    OverBorrowableAmount,

    #[msg("Amount exceeds depositable amount hence overpaying")]
    OverRepay,

    #[msg("User is not under collateralized, hence cannot be liquidated")]
    NotUnderCollateralized,
}
