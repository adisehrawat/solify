use anchor_lang::prelude::*;
use {
    mollusk_svm::Mollusk,
    solana_sdk::{account::Account, instruction::{AccountMeta, Instruction}, pubkey::Pubkey},
};
use std::str::FromStr;

// Program ID
const PROGRAM_ID: &str = "67GqHdXxaRL3SYuRn29tzbRjMJCbNxaCAyaZpKNXu76b";

fn initialize_user_discriminator() -> [u8; 8] {
    [0x6f, 0x1c, 0x1c, 0x8c, 0x3f, 0x8e, 0xa1, 0x5e]
}

fn find_user_config_pda(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"user_config", authority.as_ref()], program_id)
}

#[test]
fn test_initialize_user() {
    println!("\n=== Testing Initialize User with Mollusk ===\n");

    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();

    let system_program_id = Pubkey::from_str(&system_program::ID.to_string()).unwrap();

    let mollusk = Mollusk::new(&program_id, "../../target/deploy/solify");

    let user = Pubkey::new_unique();
    println!("User pubkey: {}", user);

    let (user_pda, bump) = find_user_config_pda(&user, &program_id);
    println!("User PDA: {}", user_pda);
    println!("Bump: {}", bump);

    let instruction_data = initialize_user_discriminator().to_vec();

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user_pda, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(system_program_id, false),
        ],
        data: instruction_data,
    };

    let mut user_account = Account::new(10_000_000_000, 0, &system_program_id);
    let pda_account = Account::default();
    let system_program_account = Account::default();

    let result: mollusk_svm::result::InstructionResult = mollusk.process_and_validate_instruction(
        &instruction,
        &vec![
            (user_pda, pda_account),
            (user, user_account),
            (system_program_id, system_program_account),
        ],
        &[],
    );

    println!("Program result: {:?}", result.program_result);
    println!("✓ Test completed");
}

// #[test]
// fn test_initialize_user_simple() {
//     println!("\n=== Simple Initialize User Test ===\n");

//     let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
//     let mollusk = Mollusk::new(&program_id, "target/deploy/solify");

//     let user = Pubkey::new_unique();
//     let (user_pda, _) = find_user_config_pda(&user, &program_id);

//     println!("User: {}", user);
//     println!("User PDA: {}", user_pda);

//     // Simple instruction test
//     let data = initialize_user_discriminator().to_vec();
//     let instruction = Instruction {
//         program_id,
//         accounts: vec![
//             AccountMeta::new(user_pda, false),
//             AccountMeta::new(user, true),
//             AccountMeta::new_readonly(system_program::ID, false),
//         ],
//         data,
//     };

//     let accounts = vec![
//         (user_pda, AccountSharedData::default()),
//         (user, AccountSharedData::new(10_000_000_000, 0, &system_program::ID)),
//         (system_program::ID, AccountSharedData::default()),
//     ];

//     let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[]);

//     println!("\nProgram Result: {:?}", result.program_result);
//     println!("✓ Test completed")
// }

// #[test]
// fn test_program_loads() {
//     println!("\n=== Testing Program Loads ===\n");

//     let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
    
//     // This will panic if the program doesn't load correctly
//     let mollusk = Mollusk::new(&program_id, "target/deploy/solify");
    
//     println!("✓ Program loaded successfully");
//     println!("Program ID: {}", program_id);
// }
