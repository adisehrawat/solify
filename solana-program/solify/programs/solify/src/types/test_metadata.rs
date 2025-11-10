use anchor_lang::prelude::*;
use super::dependencies::*;

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub instruction_order: Vec<String>,
    pub account_dependencies: Vec<AccountDependency>,
    pub pda_init_sequence: Vec<PdaInit>,
    pub setup_requirements: Vec<SetupRequirement>,
    pub test_cases: Vec<InstructionTestCases>,
}

// Simplified event-safe version
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct TestMetadataEvent {
    pub instruction_order: Vec<String>,
    pub account_dependencies: Vec<AccountDependency>,
    pub pda_init_sequence: Vec<PdaInit>,
    pub setup_requirements: Vec<SetupRequirement>,
    pub test_cases: Vec<InstructionTestCasesEvent>,
}

impl TestMetadata {
    pub fn to_event(&self) -> TestMetadataEvent {
        TestMetadataEvent {
            instruction_order: self.instruction_order.clone(),
            account_dependencies: self.account_dependencies.clone(),
            pda_init_sequence: self.pda_init_sequence.clone(),
            setup_requirements: self.setup_requirements.clone(),
            test_cases: self.test_cases.iter().map(|tc| {
                InstructionTestCasesEvent {
                    instruction_name: tc.instruction_name.clone(),
                    arguments: tc.arguments.iter().map(|arg| {
                        ArgumentInfoEvent {
                            name: arg.name.clone(),
                            arg_type_name: arg.arg_type.to_string(),
                            is_optional: arg.is_optional,
                        }
                    }).collect(),
                    positive_case_count: tc.positive_cases.len() as u32,
                    negative_case_count: tc.negative_cases.len() as u32,
                }
            }).collect(),
        }
    }
}