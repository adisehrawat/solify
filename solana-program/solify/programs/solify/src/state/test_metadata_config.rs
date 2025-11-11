use anchor_lang::prelude::*;

use crate::types::test_metadata::TestMetadata;

#[account]
#[derive(InitSpace, Debug)]
pub struct TestMetadataConfig {
    pub authority: Pubkey,
    pub program_id: Pubkey,
    #[max_len(32)]
    pub program_name: String,
    pub test_metadata: TestMetadata,
    pub timestamp: i64,
}

impl TestMetadataConfig {
    pub fn initialize(&mut self, authority: Pubkey, program_id: Pubkey, program_name: String, test_metadata: TestMetadata, timestamp: i64) -> Result<()> {
        self.authority = authority;
        self.program_id = program_id;
        self.program_name = program_name;
        self.test_metadata = test_metadata;
        self.timestamp = timestamp;
        Ok(())
    }
}