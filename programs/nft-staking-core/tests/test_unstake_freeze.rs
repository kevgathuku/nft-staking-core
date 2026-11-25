mod common;

use common::*;

// Unstaking before the 7-day freeze elapses must fail.
#[test]
fn test_unstake_before_freeze_fails() {
    let mut f = setup_initialized();
    do_stake(&mut f);
    let ix = unstake_ix(&f.acc);
    let err = send(&mut f.svm, &[&f.acc.payer], ix).unwrap_err();
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains("FreezePeriodNotElapsed"),
        "expected FreezePeriodNotElapsed, got: {err_str}"
    );
}
