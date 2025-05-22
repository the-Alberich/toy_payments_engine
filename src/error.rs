use thiserror::Error;
use crate::model::TransactionType;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Decimal error: {0}")]
    Decimal(#[from] rust_decimal::Error),

    #[error("Account Not Found. Account for client {client_id} not found. Transaction Type: {transaction_type}.")]
    AccountNotFound{client_id: u16, transaction_type: TransactionType},

    #[error("Transaction Not Found. Transaction ID: {transaction_id}. Transaction Type: {transaction_type}.")]
    TransactionNotFound{transaction_id: u32, transaction_type: TransactionType},
}
