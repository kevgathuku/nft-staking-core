mod common;

use common::*;

#[test]
fn test_create_collection() {
    let mut f = setup();
    let ix = create_collection_ix(&f.acc, "Test Collection", "https://example.com/collection");
    send(&mut f.svm, &[&f.acc.payer, &f.acc.collection], ix)
        .expect("create collection should succeed");
    assert!(
        f.svm.get_account(&f.acc.collection.pubkey()).is_some(),
        "collection account should exist"
    );
}
