use anchor_lang::prelude::*;

use crate::state::program_history::ProgramTestHistory;

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


impl UserConfig {
    pub fn initialize(&mut self, authority: Pubkey, bump: u8, timestamp: i64) {
        self.authority = authority;
        self.total_tests_generated = 0;
        self.programs_tested = Vec::new();
        self.created_at = timestamp;
        self.last_generated_at = 0;
        self.bump = bump;
    }

    pub fn update_after_generation(
        &mut self,
        program_id: String,
        program_name: String,
        idl_hash: [u8; 32],
        timestamp: i64,
    ) {
        self.total_tests_generated += 1;
        self.last_generated_at = timestamp;

        let idl_hash_string = idl_hash.to_vec().iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("");
        
        if let Some(program_history) = self.programs_tested
            .iter_mut()
            .find(|p| p.program_id == program_id)
        {
            
            program_history.test_count += 1;
            program_history.last_generated_at = timestamp;
            program_history.idl_hash = idl_hash_string;
        } else {
            self.programs_tested.push(ProgramTestHistory {
                program_id,
                program_name,
                test_count: 1,
                last_generated_at: timestamp,
                idl_hash: idl_hash_string,
            });
        }
    }
}