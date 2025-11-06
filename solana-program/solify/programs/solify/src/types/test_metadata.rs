use anchor_lang::prelude::{borsh::{BorshDeserialize, BorshSerialize}, *};
use super::dependencies::*;

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub instruction_order: Vec<String>,
    pub account_dependencies: Vec<AccountDependency>,
    pub pda_init_sequence: Vec<PdaInit>,
    pub setup_requirements: Vec<SetupRequirement>,
    pub test_cases: Vec<InstructionTestCases>,
}