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
use solana_program::hash::hash;
use crate::types::{IdlData, TestMetadata};
use crate::error::SolifyError;

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_dependencies(
        &self,
        idl_data: &IdlData,
        execution_order: &[String],
    ) -> Result<TestMetadata> {
        msg!("Starting dependency analysis...");

        // Build account registry
        let dependency_analyzer = DependencyAnalyzerImpl;
        let account_registry = dependency_analyzer.build_account_registry(idl_data)?;
        msg!("Account registry built with {} accounts", account_registry.accounts.len());

        // Build dependency graph
        let dependency_graph = dependency_analyzer.build_dependency_graph(
            idl_data, 
            execution_order, 
            &account_registry
        )?;
        msg!("Dependency graph built with {} nodes and {} edges", 
             dependency_graph.nodes.len(), dependency_graph.edges.len());

        // Generate account dependencies
        let account_order = AccountOrder;
        let account_dependencies = account_order.generate_account_dependencies(
            &dependency_graph, 
            &account_registry
        )?;
        msg!("Generated {} account dependencies", account_dependencies.len());

        // Validate account flow
        account_order.validate_account_flow(&account_dependencies)?;
        msg!("Account flow validation passed");

        // Detect PDAs and generate initialization sequence
        let pda_detector = PdaDetector;
        let program_id = Pubkey::default(); // This should be the target program ID
        let pda_init_sequence = pda_detector.detect_pdas(&account_registry, program_id)?;
        msg!("Detected {} PDAs", pda_init_sequence.len());

        // Generate setup requirements
        let setup_generator = SetupGenerator;
        let setup_requirements = setup_generator.generate_setup_requirements(&account_dependencies)?;
        msg!("Generated {} setup requirements", setup_requirements.len());

        // Validate setup flow
        setup_generator.validate_setup_flow(&setup_requirements)?;
        msg!("Setup flow validation passed");

        // Generate test cases
        let test_case_generator = TestCaseGenerator;
        let test_cases = test_case_generator.generate_test_cases(idl_data, execution_order)?;
        
        let total_positive_cases: usize = test_cases.iter().map(|tc| tc.positive_cases.len()).sum();
        let total_negative_cases: usize = test_cases.iter().map(|tc| tc.negative_cases.len()).sum();
        
        msg!("Generated {} test cases ({} positive, {} negative)", 
             test_cases.len(), total_positive_cases, total_negative_cases);

        Ok(TestMetadata {
            instruction_order: execution_order.to_vec(),
            account_dependencies,
            pda_init_sequence,
            setup_requirements,
            test_cases,
        })
    }
}

// Hash function for IDL data to track versions
pub fn hash_idl_data(idl_data: &IdlData) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(idl_data.name.as_bytes());
    data.extend_from_slice(idl_data.version.as_bytes());
    data.extend_from_slice(&(idl_data.instructions.len() as u32).to_le_bytes());
    data.extend_from_slice(&(idl_data.accounts.len() as u32).to_le_bytes());
    data.extend_from_slice(&(idl_data.types.len() as u32).to_le_bytes());
    
    let hash_result = hash(&data);
    hash_result.to_bytes()
}