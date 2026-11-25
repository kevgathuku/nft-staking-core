mod common;

use common::*;

#[test]
fn test_stake() {
    let mut f = setup_initialized();
    do_stake(&mut f);
    assert!(
        f.svm.get_account(&f.acc.nft.pubkey()).is_some(),
        "nft account should still exist after staking"
    );
}
