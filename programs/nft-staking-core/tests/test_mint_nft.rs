mod common;

use common::*;

#[test]
fn test_mint_nft() {
    let mut f = setup();
    let ix = create_collection_ix(&f.acc, "Test Collection", "https://example.com/collection");
    send(&mut f.svm, &[&f.acc.payer, &f.acc.collection], ix)
        .expect("create collection should succeed");
    let ix = mint_nft_ix(&f.acc, "Test NFT", "https://example.com/nft");
    send(&mut f.svm, &[&f.acc.payer, &f.acc.nft], ix).expect("mint should succeed");
    assert!(
        f.svm.get_account(&f.acc.nft.pubkey()).is_some(),
        "nft account should exist"
    );
}
