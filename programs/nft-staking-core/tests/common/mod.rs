#![allow(dead_code)]

// LiteSVM harness for nft-staking-core. No Anchor client types are used here
// on purpose: the program is Anchor 0.32 (solana 2.x types) while LiteSVM
// 0.16 is solana 3.x era, so instructions are built by hand. Field order
// matches the #[derive(Accounts)] structs; discriminators are copied from
// target/idl/nft_staking_core.json.

use std::str::FromStr;

use litesvm::LiteSVM;
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
pub use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const POINTS_PER_DAY: u32 = 10_000_000;
pub const FREEZE_DAYS: u8 = 7;
pub const WARP_DAYS: u64 = 8;

// Well-known program ids (never change).
fn program_id() -> Pubkey {
    Pubkey::from_str("HNSdczknKejMioJg5CtppPS7MHf8e7PK4wuPFhuJ5vBd").unwrap()
}
fn mpl_core_id() -> Pubkey {
    Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap()
}
fn system_program_id() -> Pubkey {
    Pubkey::from_str("11111111111111111111111111111111").unwrap()
}
fn token_program_id() -> Pubkey {
    Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
}
fn ata_program_id() -> Pubkey {
    Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap()
}

// Discriminators copied from target/idl/nft_staking_core.json.
const DISC_CREATE_COLLECTION: [u8; 8] = [156, 251, 92, 54, 233, 2, 16, 82];
const DISC_MINT_NFT: [u8; 8] = [211, 57, 6, 167, 15, 219, 35, 251];
const DISC_INIT_CONFIG: [u8; 8] = [208, 127, 21, 1, 194, 190, 196, 70];
const DISC_STAKE: [u8; 8] = [206, 176, 202, 18, 200, 209, 179, 108];
const DISC_UNSTAKE: [u8; 8] = [90, 95, 107, 42, 205, 124, 50, 225];

fn borsh_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

pub struct Accounts {
    pub payer: Keypair,
    pub collection: Keypair,
    pub nft: Keypair,
    pub update_authority: Pubkey,
    pub config: Pubkey,
    pub rewards_mint: Pubkey,
    pub user_rewards_ata: Pubkey,
}

pub struct Fixture {
    pub svm: LiteSVM,
    pub acc: Accounts,
}

fn pda(seeds: &[&[u8]], pid: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, pid).0
}

pub fn setup() -> Fixture {
    let pid = program_id();
    let payer = Keypair::new();
    let collection = Keypair::new();
    let nft = Keypair::new();
    let update_authority = pda(
        &[b"update_authority", collection.pubkey().as_ref()],
        &pid,
    );
    let config = pda(&[b"config", collection.pubkey().as_ref()], &pid);
    let rewards_mint = pda(&[b"rewards", config.as_ref()], &pid);
    let user_rewards_ata = pda(
        &[
            payer.pubkey().as_ref(),
            token_program_id().as_ref(),
            rewards_mint.as_ref(),
        ],
        &ata_program_id(),
    );

    let mut svm = LiteSVM::new();
    svm.add_program(
        pid,
        include_bytes!(concat!(
            env!("CARGO_TARGET_TMPDIR"),
            "/../deploy/nft_staking_core.so"
        )),
    )
    .expect("add program");
    svm.add_program(
        mpl_core_id(),
        include_bytes!("../../../../tests/fixtures/mpl_core.so"),
    )
    .expect("add mpl-core");
    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    Fixture {
        svm,
        acc: Accounts {
            payer,
            collection,
            nft,
            update_authority,
            config,
            rewards_mint,
            user_rewards_ata,
        },
    }
}

#[allow(clippy::result_large_err)]
pub fn send(
    svm: &mut LiteSVM,
    signers: &[&Keypair],
    ix: Instruction,
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let payer = signers[0].pubkey();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

pub fn create_collection_ix(acc: &Accounts, name: &str, uri: &str) -> Instruction {
    let mut data = DISC_CREATE_COLLECTION.to_vec();
    borsh_str(&mut data, name);
    borsh_str(&mut data, uri);
    Instruction::new_with_bytes(
        program_id(),
        &data,
        vec![
            AccountMeta::new(acc.payer.pubkey(), true),
            AccountMeta::new(acc.collection.pubkey(), true),
            AccountMeta::new_readonly(acc.update_authority, false),
            AccountMeta::new_readonly(system_program_id(), false),
            AccountMeta::new_readonly(mpl_core_id(), false),
        ],
    )
}

pub fn mint_nft_ix(acc: &Accounts, name: &str, uri: &str) -> Instruction {
    let mut data = DISC_MINT_NFT.to_vec();
    borsh_str(&mut data, name);
    borsh_str(&mut data, uri);
    Instruction::new_with_bytes(
        program_id(),
        &data,
        vec![
            AccountMeta::new(acc.payer.pubkey(), true),
            AccountMeta::new(acc.nft.pubkey(), true),
            AccountMeta::new(acc.collection.pubkey(), false),
            AccountMeta::new_readonly(acc.update_authority, false),
            AccountMeta::new_readonly(system_program_id(), false),
            AccountMeta::new_readonly(mpl_core_id(), false),
        ],
    )
}

pub fn init_config_ix(acc: &Accounts, points_per_day: u32, freeze_days: u8) -> Instruction {
    let mut data = DISC_INIT_CONFIG.to_vec();
    data.extend_from_slice(&points_per_day.to_le_bytes());
    data.push(freeze_days);
    Instruction::new_with_bytes(
        program_id(),
        &data,
        vec![
            AccountMeta::new(acc.payer.pubkey(), true),
            AccountMeta::new_readonly(acc.collection.pubkey(), false),
            AccountMeta::new_readonly(acc.update_authority, false),
            AccountMeta::new(acc.config, false),
            AccountMeta::new(acc.rewards_mint, false),
            AccountMeta::new_readonly(system_program_id(), false),
            AccountMeta::new_readonly(token_program_id(), false),
        ],
    )
}

pub fn stake_ix(acc: &Accounts) -> Instruction {
    Instruction::new_with_bytes(
        program_id(),
        &DISC_STAKE,
        vec![
            AccountMeta::new(acc.payer.pubkey(), true),
            AccountMeta::new_readonly(acc.update_authority, false),
            AccountMeta::new_readonly(acc.config, false),
            AccountMeta::new(acc.nft.pubkey(), false),
            AccountMeta::new(acc.collection.pubkey(), false),
            AccountMeta::new_readonly(mpl_core_id(), false),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
    )
}

pub fn unstake_ix(acc: &Accounts) -> Instruction {
    Instruction::new_with_bytes(
        program_id(),
        &DISC_UNSTAKE,
        vec![
            AccountMeta::new(acc.payer.pubkey(), true),
            AccountMeta::new_readonly(acc.update_authority, false),
            AccountMeta::new_readonly(acc.config, false),
            AccountMeta::new(acc.rewards_mint, false),
            AccountMeta::new(acc.user_rewards_ata, false),
            AccountMeta::new(acc.nft.pubkey(), false),
            AccountMeta::new(acc.collection.pubkey(), false),
            AccountMeta::new_readonly(mpl_core_id(), false),
            AccountMeta::new_readonly(system_program_id(), false),
            AccountMeta::new_readonly(token_program_id(), false),
            AccountMeta::new_readonly(ata_program_id(), false),
        ],
    )
}

// Setup plus create-collection plus mint plus init-config: the prelude for
// any test that acts on a live collection.
pub fn setup_initialized() -> Fixture {
    let mut f = setup();
    let ix = create_collection_ix(&f.acc, "Test Collection", "https://example.com/collection");
    send(&mut f.svm, &[&f.acc.payer, &f.acc.collection], ix).expect("create should succeed");
    let ix = mint_nft_ix(&f.acc, "Test NFT", "https://example.com/nft");
    send(&mut f.svm, &[&f.acc.payer, &f.acc.nft], ix).expect("mint should succeed");
    let ix = init_config_ix(&f.acc, POINTS_PER_DAY, FREEZE_DAYS);
    send(&mut f.svm, &[&f.acc.payer], ix).expect("init should succeed");
    f
}

pub fn do_stake(f: &mut Fixture) {
    let ix = stake_ix(&f.acc);
    send(&mut f.svm, &[&f.acc.payer], ix).expect("stake should succeed");
}

// ~400ms per slot: move the on-chain clock forward by `days`.
pub fn warp_days(svm: &mut LiteSVM, days: u64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.slot += days * 86_400 * 1_000 / 400;
    clock.unix_timestamp += (days * 86_400) as i64;
    svm.set_sysvar(&clock);
}

// Raw SPL TokenAccount layout: amount is u64 at bytes 64..72.
pub fn rewards_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    let acc = svm.get_account(ata).expect("rewards ata missing");
    u64::from_le_bytes(acc.data[64..72].try_into().unwrap())
}
