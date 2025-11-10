use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserConfig {
    pub authority: Pubkey,
    pub total_tests_generated: u64,
    pub created_at: i64,
    pub last_generated_at: i64,
    pub bump: u8,
}


impl UserConfig {
    pub fn initialize(&mut self, authority: Pubkey, bump: u8, timestamp: i64) {
        self.authority = authority;
        self.total_tests_generated = 0;
        self.created_at = timestamp;
        self.last_generated_at = 0;
        self.bump = bump;
    }

    pub fn update_after_generation(
        &mut self,
        timestamp: i64
    ) {
        self.total_tests_generated += 1;
        self.last_generated_at = timestamp;
    }
}