use std::path::PathBuf;
use anchor_lang::{AnchorDeserialize, InstructionData, system_program};
use litesvm::LiteSVM;
use solana_sdk::{
    pubkey, 
    signature::Keypair, 
    signer::Signer, 
    transaction::Transaction, 
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use anchor_lang::prelude::Pubkey as AnchorPubkey;

use crate::{
    state::{UserConfig, IdlStorage, TestMetadataConfig}
};
use std::io::{Write, BufWriter};
use std::fs::File;


use crate::types::IdlData;

pub mod parse_idl;
pub use parse_idl::*;

pub mod parsed_idl;

const PROGRAM_ID: Pubkey = pubkey!("4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb");



fn system_program_id() -> Pubkey {
    Pubkey::new_from_array(system_program::ID.to_bytes())
}

fn setup_test_environment() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let user = Keypair::new();
    let user_pubkey = user.pubkey();

    // Airdrop SOL to user
    svm.airdrop(&user_pubkey, 10_000_000_000).unwrap();
    
    // Load and add the program
    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/solify.so");
    let program_data = std::fs::read(so_path).expect("Failed to read program data");
    svm.add_program(PROGRAM_ID, program_data.as_slice()).unwrap();
    
    (svm, user)
}

fn get_user_pda(authority: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"user_config", authority.as_ref()],
        &PROGRAM_ID
    );
    pda
}

fn get_idl_storage_pda(program_id: &Pubkey, authority: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"idl_storage", program_id.as_ref(), authority.as_ref()],
        &PROGRAM_ID
    );
    pda
}

fn get_test_metadata_pda(program_id: &Pubkey, authority: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"tests_metadata", program_id.as_ref(), authority.as_ref()],
        &PROGRAM_ID
    );
    pda
}

fn create_test_idl_data() -> IdlData {
    let idl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/idls/journal.json");
    let idl_data = parse_idl(idl_path).unwrap();
    idl_data
}

#[test]
fn should_initialize_user() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();
    
    let user_pda = get_user_pda(&user_pubkey);
    
    let accounts = vec![
        AccountMeta::new(user_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let data = crate::instruction::InitializeUser {}.data();
    
    let instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    
    let result = svm.send_transaction(transaction);
    assert!(result.is_ok(), "Failed to initialize user: {:?}", result);
    

    let user_config_data = svm.get_account(&user_pda).unwrap();
    let mut data_slice = &user_config_data.data[8..];
    let user_config = UserConfig::deserialize(&mut data_slice).unwrap();
    
    let anchor_user_pubkey_check = AnchorPubkey::new_from_array(user_pubkey.to_bytes());
    assert_eq!(user_config.authority, anchor_user_pubkey_check);
    assert_eq!(user_config.total_tests_generated, 0);
    assert_eq!(user_config.last_generated_at, 0);
        
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/output1.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Test 1: should_initialize_user").unwrap();
    writeln!(file, "  User initialized successfully").unwrap();
    writeln!(file, "  User config: {:?}", user_pda).unwrap();
    writeln!(file, "  User config program history: {:#?}", user_config.program_history).unwrap();
    writeln!(file, "  Authority: {}", user_config.authority).unwrap();
    writeln!(file, "  Total tests generated: {}", user_config.total_tests_generated).unwrap();
    writeln!(file, "--------------------------------").unwrap();
    file.flush().unwrap();
}

#[test]
fn should_store_idl_data() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();
    
    let user_pda = get_user_pda(&user_pubkey);
    
    let init_accounts = vec![
        AccountMeta::new(user_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    let init_data = crate::instruction::InitializeUser {}.data();
    let init_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: init_accounts,
        data: init_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let init_tx = Transaction::new_signed_with_payer(
        &[init_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    svm.send_transaction(init_tx.clone()).unwrap();

    let test_program_id = pubkey!("4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data();
    let anchor_test_program_id = AnchorPubkey::new_from_array(test_program_id.to_bytes());
    let accounts = vec![
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let data = crate::instruction::StoreIdlData {
        idl_data: idl_data.clone(),
        program_id: anchor_test_program_id,
    }.data();
    
    let instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    
    let result = svm.send_transaction(transaction);
    assert!(result.is_ok(), "Failed to store IDL data: {:?}", result);
    
    // Verify the IDL storage account
    let idl_storage_data = svm.get_account(&idl_storage_pda).unwrap();
    let mut data_slice = &idl_storage_data.data[8..]; // Skip discriminator
    let idl_storage = IdlStorage::deserialize(&mut data_slice).unwrap();

    // Write output to tests/output.txt
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/output2.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Test 2: should_store_idl_data").unwrap();
    writeln!(file, "  IDL data stored successfully").unwrap();
    writeln!(file, "  IDL storage pda: {:?}", idl_storage_pda).unwrap();
    writeln!(file, "  Program ID: {}", idl_storage.program_id).unwrap();
    writeln!(file, "  IDL storage: {:#?}", idl_storage).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}

#[test]
fn should_generate_metadata() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();
    
    let user_pda = get_user_pda(&user_pubkey);
    
    let init_accounts = vec![
        AccountMeta::new(user_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    let init_data = crate::instruction::InitializeUser {}.data();
    let init_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: init_accounts,
        data: init_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let init_tx = Transaction::new_signed_with_payer(
        &[init_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    svm.send_transaction(init_tx.clone()).unwrap();

    let test_program_id = pubkey!("4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data();
    let anchor_test_program_id = AnchorPubkey::new_from_array(test_program_id.to_bytes());
    let store_accounts = vec![
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let store_data = crate::instruction::StoreIdlData {
        idl_data: idl_data.clone(),
        program_id: anchor_test_program_id,
    }.data();
    
    let store_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: store_accounts,
        data: store_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let store_tx = Transaction::new_signed_with_payer(
        &[store_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    svm.send_transaction(store_tx.clone()).unwrap();
    
    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey);
    let program_name = "journal".to_string();
    let execution_order = vec![
        "create_journal_entry".to_string(),
        "update_journal_entry".to_string(),
        "delete_journal_entry".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(user_pda, false),
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
    }.data();
    
    let gen_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: gen_accounts,
        data: gen_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let gen_tx = Transaction::new_signed_with_payer(
        &[gen_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    
    let result = svm.send_transaction(gen_tx);
    assert!(result.is_ok(), "Failed to generate metadata: {:?}", result);
    
    let test_metadata_data = svm.get_account(&test_metadata_pda).unwrap();
    let mut data_slice = &test_metadata_data.data[8..]; // Skip discriminator
    let test_metadata_config = TestMetadataConfig::deserialize(&mut data_slice).unwrap();
    
    
    let user_config_data = svm.get_account(&user_pda).unwrap();
    let mut user_data_slice = &user_config_data.data[8..];
    let user_config = UserConfig::deserialize(&mut user_data_slice).unwrap();
    

    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/output3.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Test 3: should_generate_metadata").unwrap();
    writeln!(file, "  Metadata generated successfully").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file, "  User config: {:#?}", user_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}

#[test]
fn should_update_idl_and_generate_metadata() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();
    
    let user_pda = get_user_pda(&user_pubkey);
    
    let init_accounts = vec![
        AccountMeta::new(user_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    let init_data = crate::instruction::InitializeUser {}.data();
    let init_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: init_accounts,
        data: init_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let init_tx = Transaction::new_signed_with_payer(
        &[init_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    svm.send_transaction(init_tx.clone()).unwrap();
    
    let test_program_id = pubkey!("4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let initial_idl_data = create_test_idl_data();
    let anchor_test_program_id = AnchorPubkey::new_from_array(test_program_id.to_bytes());
    
    let store_accounts = vec![
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let store_data = crate::instruction::StoreIdlData {
        idl_data: initial_idl_data.clone(),
        program_id: anchor_test_program_id,
    }.data();
    
    let store_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: store_accounts,
        data: store_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let store_tx = Transaction::new_signed_with_payer(
        &[store_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    svm.send_transaction(store_tx.clone()).unwrap();
    
    let mut updated_idl_data = initial_idl_data.clone();
    updated_idl_data.version = "0.2.0".to_string();
    if !updated_idl_data.instructions.is_empty() {
        updated_idl_data.instructions[0].docs = vec!["Updated doc".to_string()];
    }
    
    let update_accounts = vec![
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let update_data = crate::instruction::UpdateIdlData {
        idl_data: updated_idl_data.clone(),
        program_id: anchor_test_program_id,
    }.data();
    
    let update_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: update_accounts,
        data: update_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let update_tx = Transaction::new_signed_with_payer(
        &[update_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    
    let update_result = svm.send_transaction(update_tx.clone());
    assert!(update_result.is_ok(), "Failed to update IDL data: {:?}", update_result);
    
    let updated_idl_storage_data = svm.get_account(&idl_storage_pda).unwrap();
    let mut data_slice = &updated_idl_storage_data.data[8..];
    let idl_storage = IdlStorage::deserialize(&mut data_slice).unwrap();
    
    assert_eq!(idl_storage.idl_data.version, "0.2.0");
    assert_eq!(idl_storage.idl_data.instructions.len(), 3); 
    assert_eq!(idl_storage.idl_data.instructions[0].docs, vec!["Updated doc".to_string()]);

    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey);
    let program_name = "journal".to_string();
    let execution_order = vec![
        "create_journal_entry".to_string(),
        "update_journal_entry".to_string(),
        "delete_journal_entry".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(user_pda, false),
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
    }.data();
    
    let gen_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: gen_accounts,
        data: gen_data,
    };
    
    let recent_blockhash = svm.latest_blockhash();
    let gen_tx = Transaction::new_signed_with_payer(
        &[gen_instruction],
        Some(&user_pubkey),
        &[&user],
        recent_blockhash,
    );
    
    let result = svm.send_transaction(gen_tx);
    assert!(result.is_ok(), "Failed to generate metadata: {:?}", result);
    
    let test_metadata_data = svm.get_account(&test_metadata_pda).unwrap();
    let mut data_slice = &test_metadata_data.data[8..];
    let test_metadata_config = TestMetadataConfig::deserialize(&mut data_slice).unwrap();
    
    let user_config_data = svm.get_account(&user_pda).unwrap();
    let mut user_data_slice = &user_config_data.data[8..];
    let user_config = UserConfig::deserialize(&mut user_data_slice).unwrap();
    
    assert_eq!(user_config.total_tests_generated, 1);
    assert!(user_config.last_generated_at >= 0); 
    assert_eq!(user_config.program_history.len(), 1);
    
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/output4.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Update IDL and Generate Metadata Test").unwrap();
    writeln!(file, "  =======================================").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "  Updated IDL version: {}", idl_storage.idl_data.version).unwrap();
    writeln!(file, "  Updated IDL instructions count: {}", idl_storage.idl_data.instructions.len()).unwrap();
    writeln!(file, "  Instructions: {:?}", idl_storage.idl_data.instructions.iter().map(|i| &i.name).collect::<Vec<_>>()).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "  Metadata generated successfully").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "  User config: {:#?}", user_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();
    
    file.flush().unwrap();
}
