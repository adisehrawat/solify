pub use solify_common::types::{IdlData, TestMetadata};
pub use solify_common::errors::{SolifyError, Result};

pub mod dependency_analyzer;
pub use dependency_analyzer::*;
pub mod account_order;
pub use account_order::*;
pub mod pda_detector;
pub use pda_detector::*;
pub mod setup_generator;
pub use setup_generator::*;
pub mod test_case_generator;
pub use test_case_generator::*;
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
        println!("Starting dependency analysis...");

        // Build account registry
        let dependency_analyzer = DependencyAnalyzerImpl;
        let account_registry = dependency_analyzer.build_account_registry(idl_data, &program).map_err(|e| SolifyError::DependencyAnalysisFailed(e.to_string()))?;
        println!("Account registry built with {} accounts", account_registry.accounts.len());
        println!("Account registry: {:#?}", account_registry);

        // Build dependency graph
        let dependency_graph = dependency_analyzer.build_dependency_graph(
            idl_data, 
            execution_order, 
            &account_registry
        ).map_err(|e| SolifyError::DependencyAnalysisFailed(e.to_string()))?;

        println!("Dependency graph built with {} nodes and {} edges", 
             dependency_graph.nodes.len(), dependency_graph.edges.len());

        println!("Dependency graph: {:#?}", dependency_graph);

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━-------------------");
        println!("Account Order:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━--------------------");

        let account_order = AccountOrder;
        let account_dependencies = account_order.generate_account_dependencies(
            &dependency_graph, 
            &account_registry
        ).unwrap();
        println!("Generated {} account dependencies", account_dependencies.len());

        println!("Account dependencies: {:#?}", account_dependencies);

        account_order.validate_account_flow(&account_dependencies).unwrap();
        println!("Account flow validation passed");

        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━-------------------");
        println!("PDA Detection:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━--------------------");

        let pda_detector = PdaDetector;
        let program_id = program.clone(); 
        let pda_init_sequence = pda_detector.detect_pdas(&account_registry, program_id).unwrap();
        println!("Detected {} PDAs", pda_init_sequence.len());
        println!("PDA init sequence: {:#?}", pda_init_sequence);


        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━-------------------");
        println!("Setup Generation:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━--------------------");

        // // Generate setup requirements
        let setup_generator = SetupGenerator;
        let setup_requirements = setup_generator.generate_setup_requirements(&account_dependencies).unwrap();
        println!("Generated {} setup requirements", setup_requirements.len());
        println!("Setup requirements: {:#?}", setup_requirements);

        setup_generator.validate_setup_flow(&setup_requirements).unwrap();
        println!("Setup flow validation passed");


        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━-------------------");
        println!("Test Case Generation:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━--------------------");

        // // Generate test cases
        let test_case_generator = TestCaseGenerator;
        let test_cases = test_case_generator.generate_test_cases(idl_data, execution_order).unwrap();
        println!("Test cases: {:#?}", test_cases);
        
        let total_positive_cases: usize = test_cases.iter().map(|tc| tc.positive_cases.len()).sum();
        let total_negative_cases: usize = test_cases.iter().map(|tc| tc.negative_cases.len()).sum();
        
        println!("Generated {} test cases ({} positive, {} negative)", 
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
