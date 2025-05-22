use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_integration_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Runs the payments-engine binary against a sample CSV fixture
    let mut cmd = Command::cargo_bin("payments_engine")?;
    cmd.arg("tests/fixtures/basic.csv")
       .assert()
       .success()
       .stdout(predicate::str::contains("client,available,held,total,locked"))
       .stdout(predicate::str::contains("1,1.5000,0.0000,1.5000,false"))
       .stdout(predicate::str::contains("2,2.0000,0.0000,2.0000,false"))
       .stdout(predicate::str::contains("3,2.1000,5.4321,7.5321,false"))
       .stdout(predicate::str::contains("78,5.6789,0.0000,5.6789,false"));
    Ok(())
}

#[test]
fn test_cli_integration_complex() -> Result<(), Box<dyn std::error::Error>> {
    // Runs the payments-engine binary against our complex CSV fixture
    let mut cmd = Command::cargo_bin("payments_engine")?;
    cmd.arg("tests/fixtures/disputes.csv")
       .assert()
       .success()
       // Header check
       .stdout(predicate::str::contains("client,available,held,total,locked"))
       // Client 0: deposit 1.2345, then dispute -> 1.2345 held
       .stdout(predicate::str::contains("0,0.0000,1.2345,1.2345,false"))
       // Client 1: 10.0000 - 3.2500 = 6.7500
       .stdout(predicate::str::contains("1,6.7500,0.0000,6.7500,false"))
       // Client 2: deposit 5.5000, dispute+resolve -> 5.5000
       .stdout(predicate::str::contains("2,5.5000,0.0000,5.5000,false"))
       // Client 3: deposit 7.7777, dispute+chargeback -> 0.0000, locked
       .stdout(predicate::str::contains("3,0.0000,0.0000,0.0000,true"))
       // Client 4: 1.2000 - 0.2000 + 2.3456 = 3.3456
       .stdout(predicate::str::contains("4,3.3456,0.0000,3.3456,false"))
       // Client 5: failed withdrawal, then deposit 0.5000 -> 0.5000
       .stdout(predicate::str::contains("5,0.5000,0.0000,0.5000,false"));
    Ok(())
}
