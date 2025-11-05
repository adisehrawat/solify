use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserConfig {
    pub authority: Pubkey,
    pub total_tests_generated: u64,
    #[max_len(100)] 
    pub programs_tested: Vec<ProgramTestHistory>,
    pub created_at: i64,
    pub last_generated_at: i64,
    pub bump: u8,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ProgramTestHistory {
    #[max_len(64)] 
    pub program_id: String,
    #[max_len(50)]
    pub program_name: String,
    pub test_count: u32,
    pub last_generated_at: i64,
    #[max_len(32)]
    pub idl_hash: String,
}