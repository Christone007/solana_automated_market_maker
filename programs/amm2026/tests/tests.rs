use {
    anchor_spl::associated_token::{self, Create}, litesvm::LiteSVM, litesvm_token::CreateMint, solana_keypair::Keypair, solana_message::{Instruction, Message, VersionedMessage}, solana_pubkey::Pubkey, solana_signer::Signer, solana_transaction::versioned::VersionedTransaction
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &(Instruction),
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers);
    svm.send_transaction(tx)
}

// setup fn to initialize LiteSVM and create a payer keypair
fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey
) {
    let program_id = amm2026::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TEMPDIR"),
        "/../deploy/amm2026.so"
    ));

    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Create 2 mints with 6 decimal places and the maker as the authority
    // uses litesvm-token's CreateMint utility which creates the mint in the LiteSVM environment
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let config =  Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &amm2026::id());
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &amm2026::id());

    // derive the PDA for the vault ATA using the config PDA and the Mint A
    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    (svm, payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y)
}

#[test]
fn test_initialize(){
    let (mut svm, payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y) = setup();

    let instruction = create_initialize_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );

    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
fn test_deposit(){
    let (mut svm, payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y) = setup();
    let init_ix = create_initialize_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}
