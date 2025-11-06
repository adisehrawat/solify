use anchor_lang::prelude::*;

pub const MAX_PROGRAMS_TRACKED: usize = 50;
pub const MAX_INSTRUCTIONS: usize = 100;
pub const MAX_ACCOUNTS_PER_INSTRUCTION: usize = 20;
pub const MAX_TEST_CASES_PER_INSTRUCTION: usize = 50;