use payments_engine::engine::Engine;
use payments_engine::model::{TransactionRecord, TransactionType};
use rstest::rstest;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Helper to create a TransactionRecord easily
fn transaction(transaction_type: TransactionType, client_id: u16, transaction_id: u32, amount: Option<Decimal>) -> TransactionRecord {
    TransactionRecord { transaction_type: transaction_type, client_id, transaction_id: transaction_id, amount }
}

#[rstest]
fn test_deposit_then_dispute_moves_to_held() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(10.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(0.0000));
    assert_eq!(account.held, dec!(10.0000));
    assert_eq!(account.total, dec!(10.0000));
}

#[rstest]
fn test_dispute_on_nonexistent_tx_is_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 99, None)).unwrap();
    assert!(engine.accounts.is_empty());
}

#[rstest]
fn test_resolve_moves_from_held_to_available() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Resolve, 1, 1, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(5.0000));
    assert_eq!(account.held, dec!(0.0000));
    assert_eq!(account.total, dec!(5.0000));
}

#[rstest]
fn test_resolve_on_non_disputed_tx_is_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(3.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Resolve, 1, 1, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(3.0000));
    assert_eq!(account.held, dec!(0.0000));
}

#[rstest]
fn test_chargeback_locks_and_subtracts() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 2, 5, Some(dec!(8.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 2, 5, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Chargeback, 2, 5, None)).unwrap();

    let account = engine.accounts.get(&2).unwrap();
    assert!(account.locked);
    assert_eq!(account.available, dec!(0.0000));
    assert_eq!(account.held, dec!(0.0000));
    assert_eq!(account.total, dec!(0.0000));
}

#[rstest]
fn test_chargeback_on_non_disputed_tx_is_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 2, 5, Some(dec!(4.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Chargeback, 2, 5, None)).unwrap();

    let account = engine.accounts.get(&2).unwrap();
    assert!(!account.locked);
    assert_eq!(account.available, dec!(4.0000));
    assert_eq!(account.total, dec!(4.0000));
}

#[rstest]
fn test_multiple_operations_sequence() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(10.1234)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(3.2100)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 3, Some(dec!(2.5555)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 4, Some(dec!(4.3333)))).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    // 10.1234 - 3.2100 + 2.5555 - 4.3333 = 5.1356
    assert_eq!(account.available, dec!(5.1356));
    assert_eq!(account.total, dec!(5.1356));
}

#[rstest]
fn test_failed_withdrawal_then_dispute_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.5432)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(10.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 2, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    // Excess withdrawal ignored, so original available remains
    assert_eq!(account.available, dec!(5.5432));
    assert_eq!(account.held, dec!(0.0000));
}

#[rstest]
fn test_double_dispute_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(3.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(0.0000));
    assert_eq!(account.held, dec!(3.0000));
}

#[rstest]
fn test_double_resolve_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(4.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Resolve, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Resolve, 1, 1, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(4.0000));
    assert_eq!(account.held, dec!(0.0000));
}

#[rstest]
fn test_post_chargeback_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(6.7890)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Chargeback, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 2, Some(dec!(5.4321)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 3, Some(dec!(1.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Resolve, 1, 1, None)).unwrap();

    let account = engine.accounts.get(&1).unwrap();
    assert!(account.locked);
    // All operations after lock are ignored, total remains 0.0000
    assert_eq!(account.available, dec!(0.0000));
    assert_eq!(account.held, dec!(0.0000));
    assert_eq!(account.total, dec!(0.0000));
}

#[rstest]
fn test_zero_amount_deposit() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(0.0000)))).unwrap();
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(0.0000));
    assert_eq!(account.total, dec!(0.0000));
}

#[rstest]
fn test_multiple_clients_isolation() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(2.1234)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Deposit, 2, 2, Some(dec!(3.4567)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 3, Some(dec!(1.1234)))).unwrap();

    let account1 = engine.accounts.get(&1).unwrap();
    // 2.1234 - 1.1234 = 1.0000
    assert_eq!(account1.available, dec!(1.0000));
    let account2 = engine.accounts.get(&2).unwrap();
    assert_eq!(account2.available, dec!(3.4567));
}

#[rstest]
fn test_deposit_on_locked_account_is_ignored() {
    let mut engine = Engine::new();
    // Deposit, dispute, chargeback to lock account
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Chargeback, 1, 1, None)).unwrap();

    // Attempt deposit after lock
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 2, Some(dec!(3.0000)))).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert!(acct.locked);
    assert_eq!(acct.available, dec!(0.0000));
    assert_eq!(acct.total, dec!(0.0000));
}

#[rstest]
fn test_deposit_missing_amount_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, None)).unwrap();
    assert!(engine.accounts.is_empty(), "Account created on missing-amount deposit");
}

#[rstest]
fn test_withdrawal_missing_amount_ignored() {
    let mut engine = Engine::new();
    // seed with initial deposit
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, None)).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(5.0000));
    assert_eq!(acct.total, dec!(5.0000));
}

#[rstest]
fn test_duplicate_deposit_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(3.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(3.0000)))).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(3.0000));
    assert_eq!(acct.total, dec!(3.0000));
}

#[rstest]
fn test_duplicate_withdrawal_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(2.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(2.0000)))).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(3.0000));
    assert_eq!(acct.total, dec!(3.0000));
}

#[rstest]
fn test_dispute_mismatched_client_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(4.0000)))).unwrap();
    // dispute by wrong client
    engine.process_transaction(transaction(TransactionType::Dispute, 2, 1, None)).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(4.0000));
    assert_eq!(acct.held, dec!(0.0000));
}

#[rstest]
fn test_resolve_mismatched_client_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(4.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    // resolve by wrong client
    engine.process_transaction(transaction(TransactionType::Resolve, 2, 1, None)).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.held, dec!(4.0000));
    assert_eq!(acct.available, dec!(0.0000));
}

#[rstest]
fn test_chargeback_mismatched_client_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(4.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    // chargeback by wrong client
    engine.process_transaction(transaction(TransactionType::Chargeback, 2, 1, None)).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.held, dec!(4.0000));
    assert_eq!(acct.total, dec!(4.0000));
    assert!(!acct.locked, "Account should not lock on mismatched chargeback.");
}

#[rstest]
fn test_withdrawal_on_locked_account_is_ignored() {
    let mut engine = Engine::new();
    // Deposit, dispute, chargeback to lock account
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Chargeback, 1, 1, None)).unwrap();

    // Attempt withdrawal after lock
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(1.0000)))).unwrap();
    let acct = engine.accounts.get(&1).unwrap();
    assert!(acct.locked);
    assert_eq!(acct.available, dec!(0.0000));
    assert_eq!(acct.total, dec!(0.0000));
}

#[rstest]
fn test_redispute_after_resolve_allows_second_dispute() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(7.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Resolve, 1, 1, None)).unwrap();
    // Re-dispute same transaction
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();

    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(0.0000));
    assert_eq!(acct.held, dec!(7.0000));
}

#[rstest]
fn test_exact_withdrawal_zeroes_account() {
    let mut engine = Engine::new();
    // Deposit with tx=1
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    // Withdraw exact amount with a new tx id=2
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(5.0000)))).unwrap();

    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(0.0000));
    assert_eq!(acct.total, dec!(0.0000));
}

#[rstest]
fn test_duplicate_chargeback_ignored() {
    let mut engine = Engine::new();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(4.0000)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Dispute, 1, 1, None)).unwrap();
    engine.process_transaction(transaction(TransactionType::Chargeback, 1, 1, None)).unwrap();
    // Second chargeback should be skipped
    engine.process_transaction(transaction(TransactionType::Chargeback, 1, 1, None)).unwrap();

    let acct = engine.accounts.get(&1).unwrap();
    assert!(acct.locked);
    assert_eq!(acct.available, dec!(0.0000));
    assert_eq!(acct.held, dec!(0.0000));
    assert_eq!(acct.total, dec!(0.0000));
}

#[rstest]
fn test_withdrawal_reusing_deposit_transaction_id_is_ignored() {
    let mut engine = Engine::new();
    // Initial deposit with tx=1
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    // Attempt withdrawal using same tx id=1
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 1, Some(dec!(2.0000)))).unwrap();
    // Ensure deposit untouched and withdrawal not applied
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(5.0000));
    assert_eq!(acct.total, dec!(5.0000));
}

#[rstest]
fn test_deposit_reusing_withdrawal_transaction_id_is_ignored() {
    let mut engine = Engine::new();
    // Initial deposit with tx=1 to give account non-zero total / available
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(5.0000)))).unwrap();
    // Attempt withdrawal using unique tx id=2
    engine.process_transaction(transaction(TransactionType::Withdrawal, 1, 2, Some(dec!(2.0000)))).unwrap();
    // Attempt withdrawal using same tx id=2
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(2.0000)))).unwrap();
    // Ensure deposit untouched and withdrawal not applied
    let acct = engine.accounts.get(&1).unwrap();
    assert_eq!(acct.available, dec!(3.0000));
    assert_eq!(acct.total, dec!(3.0000));
}

#[rstest]
fn test_deposit_varied_decimal_precision() {
    let mut engine = Engine::new();
    // Deposit with four-digit precision then a small fraction
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 1, Some(dec!(1.2345)))).unwrap();
    engine.process_transaction(transaction(TransactionType::Deposit, 1, 2, Some(dec!(0.0001)))).unwrap();

    let acct = engine.accounts.get(&1).unwrap();
    // 1.2345 + 0.0001 = 1.2346
    assert_eq!(acct.available, dec!(1.2346));
    assert_eq!(acct.total, dec!(1.2346));
}
