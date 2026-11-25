mod common;

use common::*;

#[test]
fn test_init_config() {
    let f = setup_initialized();
    assert!(
        f.svm.get_account(&f.acc.config).is_some(),
        "config account should exist"
    );
    assert!(
        f.svm.get_account(&f.acc.rewards_mint).is_some(),
        "rewards mint should exist"
    );
}
