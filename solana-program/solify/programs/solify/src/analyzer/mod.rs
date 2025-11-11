pub mod dependency_analyzer;
pub mod pda_detector;
pub mod account_order;
pub mod setup_generator;
pub mod test_case_generator;

pub use dependency_analyzer::*;
pub use pda_detector::*;
pub use account_order::*;
pub use setup_generator::*;
pub use test_case_generator::*;

use anchor_lang::prelude::*;
use crate::types::{IdlData, TestMetadata};

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_dependencies(
        &self,
        idl_data: &IdlData,
        execution_order: &[String],
        program: String,
    ) -> Result<TestMetadata> {

        let dependency_analyzer = DependencyAnalyzerImpl;
        let account_registry = dependency_analyzer.build_account_registry(idl_data, &program.to_string())?;

        let dependency_graph = dependency_analyzer.build_dependency_graph(
            idl_data, 
            execution_order, 
            &account_registry
        )?;

        let account_order = AccountOrder;
        let account_dependencies = account_order.generate_account_dependencies(
            &dependency_graph, 
            &account_registry
        )?;

        // Validate account flow
        account_order.validate_account_flow(&account_dependencies)?;

        // Detect PDAs and generate initialization sequence
        let pda_detector = PdaDetector;
        let program_id = Pubkey::default(); // This should be the target program ID
        let pda_init_sequence = pda_detector.detect_pdas(&account_registry, program_id)?;

        // Generate setup requirements
        let setup_generator = SetupGenerator;
        let setup_requirements = setup_generator.generate_setup_requirements(&account_dependencies)?;
        // msg!("Generated {} setup requirements", setup_requirements.len());

        // Validate setup flow
        setup_generator.validate_setup_flow(&setup_requirements)?;
        // msg!("Setup flow validation passed");

        let test_case_generator = TestCaseGenerator;
        let test_cases = test_case_generator.generate_test_cases(idl_data, execution_order)?;

        Ok(TestMetadata {
            instruction_order: execution_order.to_vec(),
            account_dependencies,
            pda_init_sequence,
            setup_requirements,
            test_cases,
        })
    }
}

