use anchor_lang::prelude::*;
use super::dependencies::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct TestMetadata {
    #[max_len(3, 10)]
    pub instruction_order: Vec<String>,
    #[max_len(3)]
    pub account_dependencies: Vec<AccountDependency>,
    #[max_len(3)]
    pub pda_init_sequence: Vec<PdaInit>,
    #[max_len(3)]
    pub setup_requirements: Vec<SetupRequirement>,
    #[max_len(1)]
    pub test_cases: Vec<InstructionTestCases>,
}


