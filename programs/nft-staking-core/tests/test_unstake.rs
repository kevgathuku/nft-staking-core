mod common;

use common::*;

// 8 days staked at 10M points/day = 80M reward tokens (6 decimals).
#[test]
fn test_unstake() {
    let mut f = setup_initialized();
    do_stake(&mut f);
    warp_days(&mut f.svm, WARP_DAYS);
    let ix = unstake_ix(&f.acc);
    send(&mut f.svm, &[&f.acc.payer], ix).expect("unstake should succeed");
    assert_eq!(
        rewards_balance(&f.svm, &f.acc.user_rewards_ata),
        WARP_DAYS * POINTS_PER_DAY as u64
    );
}
