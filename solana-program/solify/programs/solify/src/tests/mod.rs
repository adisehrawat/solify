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
    state::{ TestMetadataConfig}
};
use std::io::{Write, BufWriter};
use std::fs::File;


use crate::types::IdlData;

pub mod parse_idl;
pub use parse_idl::*;

pub mod parsed_idl;

const PROGRAM_ID: Pubkey = pubkey!("7tvJ6jxJF81pozUSa2o8yPo6zsQCxG4GyF2b6JgaHqaa");



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


fn get_idl_storage_pda(program_id: &Pubkey, authority: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"idl_storage", program_id.as_ref(), authority.as_ref()],
        &PROGRAM_ID
    );
    pda
}

fn get_test_metadata_pda(program_id: &Pubkey, authority: &Pubkey, paraphrase: &str) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"tests_metadata", program_id.as_ref(), authority.as_ref(), paraphrase.as_bytes()],
        &PROGRAM_ID
    );
    pda
}

fn create_test_idl_data(path:String) -> IdlData {
    let idl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(path);
    let idl_data = parse_idl(idl_path).unwrap();
    idl_data
}


#[test]
fn test_for_idl1() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();

    let test_program_id = pubkey!("7tvJ6jxJF81pozUSa2o8yPo6zsQCxG4GyF2b6JgaHqaa");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data("src/tests/idls/journal.json".to_string());
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


    let paraphrase = "test_for_idl1";
    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey, paraphrase);
    let program_name = "journal".to_string();
    let execution_order = vec![
        "create_journal_entry".to_string(),
        "update_journal_entry".to_string(),
        "delete_journal_entry".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
        paraphrase: paraphrase.to_string(),
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
    
    
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/test_for_idl1.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Metadata generated successfully for idl1").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}


#[test]
fn test_for_idl2() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();

    let test_program_id = pubkey!("7tvJ6jxJF81pozUSa2o8yPo6zsQCxG4GyF2b6JgaHqaa");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data("src/tests/idls/counter_program.json".to_string());
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


    let paraphrase = "test_for_idl1";
    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey, paraphrase);
    let program_name = "counter_program".to_string();
    let execution_order = vec![
        "initialize".to_string(),
        "increment".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
        paraphrase: paraphrase.to_string(),
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
    
    
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/test_for_idl2.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Metadata generated successfully for idl2").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}


#[test]
fn test_for_idl3() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();

    let test_program_id = pubkey!("7tvJ6jxJF81pozUSa2o8yPo6zsQCxG4GyF2b6JgaHqaa");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data("src/tests/idls/greeting_program.json".to_string());
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


    let paraphrase = "test_for_idl1";
    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey, paraphrase);
    let program_name = "greeting_program".to_string();
    let execution_order = vec![
        "setMessage".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
        paraphrase: paraphrase.to_string(),
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
    
    
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/test_for_idl3.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Metadata generated successfully for idl3").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}


#[test]
fn test_for_idl4() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();

    let test_program_id = pubkey!("7tvJ6jxJF81pozUSa2o8yPo6zsQCxG4GyF2b6JgaHqaa");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data("src/tests/idls/mini_escrow.json".to_string());
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


    let paraphrase = "test_for_idl1";
    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey, paraphrase);
    let program_name = "mini_escrow".to_string();
    let execution_order = vec![
        "initEscrow".to_string(),
        "cancelEscrow".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
        paraphrase: paraphrase.to_string(),
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
    
    
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/test_for_idl4.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Metadata generated successfully for idl4").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}


#[test]
fn test_for_idl5() {
    let (mut svm, user) = setup_test_environment();
    let user_pubkey = user.pubkey();

    let test_program_id = pubkey!("7tvJ6jxJF81pozUSa2o8yPo6zsQCxG4GyF2b6JgaHqaa");
    let idl_storage_pda = get_idl_storage_pda(&test_program_id, &user_pubkey);
    let idl_data = create_test_idl_data("src/tests/idls/token_vault.json".to_string());
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


    let paraphrase = "test_for_idl1";
    let test_metadata_pda = get_test_metadata_pda(&test_program_id, &user_pubkey, paraphrase);
    let program_name = "token_vault".to_string();
    let execution_order = vec![
        "createVault".to_string(),
        "deposit".to_string(),
    ];
    
    let gen_accounts = vec![
        AccountMeta::new(test_metadata_pda, false),
        AccountMeta::new(idl_storage_pda, false),
        AccountMeta::new(user_pubkey, true),
        AccountMeta::new_readonly(system_program_id(), false),
    ];
    
    let gen_data = crate::instruction::GenerateMetadata {
        execution_order: execution_order.clone(),
        program_id: anchor_test_program_id,
        program_name: program_name.clone(),
        paraphrase: paraphrase.to_string(),
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
    
    
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/outputs/test_for_idl5.txt");
    let mut file = BufWriter::new(File::create(output_path).unwrap());
    writeln!(file, "  =======================================").unwrap();
    writeln!(file, "  Metadata generated successfully for idl5").unwrap();
    writeln!(file, "  Test metadata config: {:#?}", test_metadata_config).unwrap();
    writeln!(file, "--------------------------------").unwrap();

    file.flush().unwrap();
}
