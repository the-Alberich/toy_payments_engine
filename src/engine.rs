use log::{warn, error};
use crate::model::{Account, TransactionRecord, TransactionType};
use crate::error::ApplicationError;
use std::collections::{HashMap, HashSet};

pub struct Engine {
    pub accounts: HashMap<u16, Account>,
    pub transactions: HashMap<u32, TransactionRecord>,
    pub disputes: HashSet<u32>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            disputes: HashSet::new(),
        }
    }

    pub fn process_transaction(&mut self, record: TransactionRecord) -> Result<(), ApplicationError> {
        let client_id = record.client_id;
        let transaction_id = record.transaction_id;

        match record.transaction_type {
            TransactionType::Deposit => {
                // Warn and skip when Deposit transaction is missing amount.
                let amount = match record.amount {
                    Some(amount) => amount,
                    None => {
                        warn!("Deposit transaction {} missing amount. Skipping.", transaction_id);
                        return Ok(());
                    }
                };
                
                // Create account if it doesn't exist on Deposit
                let account = self.accounts.entry(client_id).or_insert_with(Account::new);

                // Warn and skip if account is locked on Deposit.
                if account.locked {
                    warn!("Deposit on locked account is not allowed for client {} in transaction {}. Skipping.", client_id, transaction_id);
                    return Ok(());
                }


                // Warn and skip if the transaction ID has already been used.
                if self.transactions.contains_key(&transaction_id) {
                    warn!("Transaction has already been processed for transaction {}. Skipping.", transaction_id);
                    return Ok(());
                }

                account.available += amount;
                account.total += amount;
                self.transactions.insert(transaction_id, record);
            }
            TransactionType::Withdrawal => {
                // Warn and skip when Withdrawal transaction is missing amount.
                let amount = match record.amount {
                    Some(amount) => amount,
                    None => {
                        warn!("Withdrawal transaction {} missing amount. Skipping.", transaction_id);
                        return Ok(());
                    }
                };

                // Warn and skip if account doesn't exist on Withdrawal.
                let account = match self.accounts.get_mut(&client_id) {
                    Some(account) => account,
                    None => {
                        warn!("Withdrawal for unknown client {} in transaction {}. Skipping.", client_id, transaction_id);
                        return Ok(());
                    }
                };

                // Warn and skip if account is locked on Withdrawal.
                if account.locked {
                    warn!("Withdrawal on locked account is not allowed for client {} in transaction {}. Skipping.", client_id, transaction_id);
                    return Ok(());
                }


                // Warn and skip if the transaction ID has already been used.
                if self.transactions.contains_key(&transaction_id) {
                    warn!("Transaction has already been processed for transaction {}. Skipping.", transaction_id);
                    return Ok(());
                }

                if account.available >= amount {
                    account.available -= amount;
                    account.total -= amount;
                    self.transactions.insert(transaction_id, record);
                }
                else {
                    warn!("Withdrawal request failed due to insufficient available funds for client {} in transaction {}. Skipping.", client_id, transaction_id);
                }
            }
            TransactionType::Dispute => {
                // Warn and skip when transaction is unknown on Dispute.
                let disputed_transaction = match self.transactions.get(&transaction_id) {
                    Some(disputed_transaction) => disputed_transaction,
                    None => {
                        warn!("Dispute on unknown transaction {}. Skipping.", transaction_id);
                        return Ok(());
                    }
                };

                // Warn and skip when transaction is already disputed on Dispute.
                if self.disputes.contains(&transaction_id) {
                    warn!("Dispute already exists for transaction {}. Skipping.", transaction_id);
                    return Ok(());
                }

                // Warn and continue for disputes that have transaction_id / client_id mismatch on Dispute.
                // Arguably this could be ignored and Dispute could be processed only using the disputed_transaction's client_id, but it represents bad data from input so skipping.
                if client_id != disputed_transaction.client_id {
                    warn!("Dispute for transaction {} has mismatched client_id. Disputed transaction client_id is {}. Dispute record client_id is {}. Skipping.", transaction_id, disputed_transaction.client_id, client_id);
                    return Ok(());
                }

                let account = match self.accounts.get_mut(&disputed_transaction.client_id) {
                    Some(account) => account,
                    None => {
                        // This shouldn’t normally happen, but guard nonetheless.
                        error!("Dispute for known transaction {}, but account is missing for client {}.", transaction_id, disputed_transaction.client_id);
                        Err(ApplicationError::AccountNotFound { client_id: client_id, transaction_type: TransactionType::Dispute })
                    }?
                };
                if let Some(amount) = disputed_transaction.amount {
                    account.available -= amount;
                    account.held += amount;
                    self.disputes.insert(transaction_id);
                }
            }
            TransactionType::Resolve => {
                // Warn and skip when dispute doesn't exist on Resolve.
                if !self.disputes.contains(&transaction_id) {
                    warn!("Resolve on non-disputed transaction {}. Skipping.", transaction_id);
                    return Ok(());
                }

                let disputed_transaction = match self.transactions.get(&transaction_id) {
                    Some(disputed_transaction) => disputed_transaction,
                    None => {
                        // This shouldn’t normally happen, but guard nonetheless.
                        error!("Resolve on unknown transaction {}, but dispute exists.", transaction_id);
                        Err(ApplicationError::TransactionNotFound { transaction_id: transaction_id, transaction_type: TransactionType::Dispute })
                    }?
                };

                // Warn and skip for resolves that have transaction_id / client_id mismatch on Resolve.
                // Arguably this could be ignored and Resolve could be processed only using the disputed_transaction's client_id, but it represents bad data from input so skipping.
                if client_id != disputed_transaction.client_id {
                    warn!("Resolve for disputed transaction {} has mismatched client_id. Disputed transaction client_id is {}. Resolve record client_id is {}. Skipping.", transaction_id, disputed_transaction.client_id, client_id);
                    return Ok(());
                }

                let account = match self.accounts.get_mut(&disputed_transaction.client_id) {
                    Some(account) => account,
                    None => {
                        // This shouldn’t normally happen, but guard nonetheless.
                        error!("Resolve for known transaction {}, but account is missing for client {}.", transaction_id, disputed_transaction.client_id);
                        Err(ApplicationError::AccountNotFound { client_id: client_id, transaction_type: TransactionType::Resolve })
                    }?
                };
                if let Some(amount) = disputed_transaction.amount {
                    account.held -= amount;
                    account.available += amount;
                    self.disputes.remove(&transaction_id);
                }
            }
            TransactionType::Chargeback => {
                // Warn and skip when dispute doesn't exist on Chargeback.
                if !self.disputes.contains(&transaction_id) {
                    warn!("Chargeback on non-disputed transaction {}. Skipping.", transaction_id);
                    return Ok(());
                }

                let disputed_transaction = match self.transactions.get(&transaction_id) {
                    Some(disputed_transaction) => disputed_transaction,
                    None => {
                        // This shouldn’t normally happen, but guard nonetheless.
                        error!("Chargeback on unknown transaction {}, but dispute exists.", transaction_id);
                        Err(ApplicationError::TransactionNotFound { transaction_id: transaction_id, transaction_type: TransactionType::Chargeback })
                    }?
                };

                // Warn and skip for chargebacks that have transaction_id / client_id mismatch on Chargeback.
                // Arguably this could be ignored and Chargeback could be processed only using the disputed_transaction's client_id, but it represents bad data from input so skipping.
                if client_id != disputed_transaction.client_id {
                    warn!("Chargeback for disputed transaction {} has mismatched client_id. Disputed transaction client_id is {}. Chargeback record client_id is {}. Skipping.", transaction_id, disputed_transaction.client_id, client_id);
                    return Ok(());
                }

                let account = match self.accounts.get_mut(&disputed_transaction.client_id) {
                    Some(account) => account,
                    None => {
                        // This shouldn’t normally happen, but guard nonetheless.
                        error!("Chargeback for known transaction {}, but account is missing for client {}.", transaction_id, disputed_transaction.client_id);
                        Err(ApplicationError::AccountNotFound { client_id: client_id, transaction_type: TransactionType::Resolve })
                    }?
                };
                if let Some(amount) = disputed_transaction.amount {
                    account.held -= amount;
                    account.total -= amount;
                    account.locked = true;
                    self.disputes.remove(&transaction_id);
                }
            }
        }
        Ok(())
    }
}
