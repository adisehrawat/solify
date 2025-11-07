use solify_common::types::{SetupRequirement, SetupType, AccountDependency};
use solify_common::errors::{SolifyError, Result};

pub struct SetupGenerator;

impl SetupGenerator {
    pub fn generate_setup_requirements(
        &self,
        account_dependencies: &[AccountDependency],
    ) -> Result<Vec<SetupRequirement>> {
        let mut setup_requirements = Vec::new();

        // Add keypair creation for signers
        let signer_accounts: Vec<_> = account_dependencies
            .iter()
            .filter(|ad| ad.is_signer && !ad.is_pda)
            .collect();

        for signer in &signer_accounts {
            setup_requirements.push(SetupRequirement {
                requirement_type: SetupType::CreateKeypair,
                description: format!("Create keypair for {}", signer.account_name),
                dependencies: Vec::new(),
            });
        }

        // Add funding requirements for signers
        for signer in signer_accounts {
            setup_requirements.push(SetupRequirement {
                requirement_type: SetupType::FundAccount,
                description: format!("Fund {} with SOL for transactions", signer.account_name),
                dependencies: vec![signer.account_name.clone()],
            });
        }

        // Add PDA initialization requirements
        let pda_accounts: Vec<_> = account_dependencies
            .iter()
            .filter(|ad| ad.is_pda && ad.must_be_initialized)
            .collect();

        for pda in pda_accounts {
            let mut dependencies = Vec::new();
            
            // Add dependencies for PDA seeds
            for dep in &pda.depends_on {
                if account_dependencies.iter().any(|ad| &ad.account_name == dep) {
                    dependencies.push(dep.clone());
                }
            }

            setup_requirements.push(SetupRequirement {
                requirement_type: SetupType::InitializePda,
                description: format!("Initialize {} PDA", pda.account_name),
                dependencies,
            });
        }

        // Sort setup requirements by dependencies
        self.sort_setup_requirements(&mut setup_requirements)?;

        Ok(setup_requirements)
    }

    fn sort_setup_requirements(&self, requirements: &mut Vec<SetupRequirement>) -> Result<()> {
        let mut graph = std::collections::HashMap::new();
        
        // Build dependency graph
        for (i, req) in requirements.iter().enumerate() {
            graph.insert(i, req.dependencies.clone());
        }

        // Topological sort
        let mut visited = std::collections::HashSet::new();
        let mut sorted_indices = Vec::new();

        for i in 0..requirements.len() {
            if !visited.contains(&i) {
                self.visit_requirement(i, &graph, requirements, &mut visited, &mut sorted_indices)?;
            }
        }

        // Reorder requirements
        let mut sorted_requirements = Vec::new();
        for &idx in &sorted_indices {
            sorted_requirements.push(requirements[idx].clone());
        }
        *requirements = sorted_requirements;

        Ok(())
    }

    fn visit_requirement(
        &self,
        index: usize,
        graph: &std::collections::HashMap<usize, Vec<String>>,
        requirements: &[SetupRequirement],
        visited: &mut std::collections::HashSet<usize>,
        sorted: &mut Vec<usize>,
    ) -> Result<()> {
        visited.insert(index);

        if let Some(dependencies) = graph.get(&index) {
            for dep_name in dependencies {
                // Find the index of the dependency requirement
                if let Some(dep_index) = graph.keys().find(|&&i| {
                    requirements[i].description.contains(dep_name.as_str())
                }).copied() {
                    if !visited.contains(&dep_index) {
                        self.visit_requirement(dep_index, graph, requirements, visited, sorted)?;
                    }
                }
            }
        }

        sorted.push(index);
        Ok(())
    }

    pub fn validate_setup_flow(&self, requirements: &[SetupRequirement]) -> Result<bool> {
        // Check that all dependencies are satisfied in the setup flow
        let mut satisfied_dependencies = std::collections::HashSet::new();

        for requirement in requirements {
            for dependency in &requirement.dependencies {
                if !satisfied_dependencies.contains(dependency) {
                    return Err(SolifyError::DependencyAnalysisFailed(format!("Dependency not satisfied: {}", dependency)))?;
                }
            }
            // Mark this requirement's target as satisfied
            if let Some(target) = self.extract_target_from_description(&requirement.description) {
                satisfied_dependencies.insert(target);
            }
        }

        Ok(true)
    }

    fn extract_target_from_description(&self, description: &str) -> Option<String> {
        // Extract the account name from setup description
        if description.contains("for ") {
            description.split("for ").nth(1).map(|s| s.to_string())
        } else if description.contains("Initialize ") {
            description.split("Initialize ").nth(1)
                .and_then(|s| s.split(' ').next())
                .map(|s| s.to_string())
        } else {
            None
        }
    }
}