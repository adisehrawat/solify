use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ProgramTestHistory {
    #[max_len(44)] 
    pub program_id: String,
    #[max_len(32)]
    pub program_name: String,
    pub test_count: u32,
    pub last_generated_at: i64,
    #[max_len(64)]
    pub idl_hash: String,
}