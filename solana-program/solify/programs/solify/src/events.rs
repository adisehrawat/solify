use anchor_lang::prelude::*;
use crate::types::test_metadata::TestMetadataEvent;

#[event]
pub struct UserProfileCreated {
    pub user: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TestGenerated {
    pub user: Pubkey,
    pub program_id: Pubkey,
    pub program_name: String,
    pub test_count: u64,
    pub instruction_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct MetadataGenerated {
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub program_name: String,
    pub idl_hash: [u8; 32],
    pub instruction_count: u32,
    pub test_case_count: u32,
    pub timestamp: i64,
}

#[event]
pub struct TestMetadataGenerated {
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub program_name: String,
    pub account_dependencies_count: u32,
    pub pda_init_count: u32,
    pub setup_requirements_count: u32,
    pub total_test_cases: u32,
    pub positive_test_cases: u32,
    pub negative_test_cases: u32,
    pub timestamp: i64,
}

#[event]
pub struct CompleteTestMetadata {
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub program_name: String,
    pub test_metadata: TestMetadataEvent,
    pub timestamp: i64,
}
