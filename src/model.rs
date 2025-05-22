use std::fmt;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// Used for stringifying TransactionType in ApplicationError messages.
impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // match on your variants and write the exact string you want
        let s = match self {
            TransactionType::Deposit    => "Deposit",
            TransactionType::Withdrawal => "Withdrawal",
            TransactionType::Dispute    => "Dispute",
            TransactionType::Resolve    => "Resolve",
            TransactionType::Chargeback => "Chargeback",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    pub amount: Option<Decimal>,
}

#[derive(Debug)]
pub struct Account {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    /// Initializes a new Account with zero balances and unlocked state
    pub fn new() -> Self {
        Account {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }
}
