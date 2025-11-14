use solify_common::types::{PdaInit, SeedComponent, SeedType as OutputSeedType};
use solify_common::errors::{SolifyError, Result};
use crate::dependency_analyzer::*;

pub struct PdaDetector;

impl PdaDetector {
    pub fn detect_pdas(&self, registry: &AccountRegistry, program_id: String) -> Result<Vec<PdaInit>> {
        let mut pda_inits = Vec::new();

        for account in &registry.accounts {
            if account.is_pda {
                let pda_init = self.create_pda_init(account, program_id.clone()).unwrap();
                pda_inits.push(pda_init);
            }
        }

        // Sort PDAs by their dependencies
        self.sort_pdas_by_dependencies(&mut pda_inits, registry).unwrap();

        Ok(pda_inits)
    }

    fn create_pda_init(&self, account: &AccountInfo, program_id: String) -> Result<PdaInit> {
        let seeds = account.seeds
            .iter()
            .map(|seed_info| {
                let seed_type = match seed_info.seed_type {
                    SeedType::Static => OutputSeedType::Static,
                    SeedType::AccountKey => OutputSeedType::AccountKey,
                    SeedType::Argument => OutputSeedType::Argument,
                };

                SeedComponent {
                    seed_type,
                    value: seed_info.value.clone(),
                }
            })
            .collect();

        // Estimate space requirement based on account usage
        let space = self.estimate_account_space(account);

        Ok(PdaInit {
            account_name: account.name.clone(),
            seeds,
            program_id: program_id.clone(),
            space: Some(space),
        })
    }

    fn estimate_account_space(&self, account: &AccountInfo) -> u64 {
        // Basic space estimation based on account type and usage patterns
        let base_space = 8; // Account discriminator
        
        match account.name.to_lowercase().as_str() {
            name if name.contains("user") || name.contains("account") => base_space + 128,
            name if name.contains("vault") => base_space + 256,
            name if name.contains("pool") => base_space + 512,
            name if name.contains("market") => base_space + 1024,
            _ => base_space + 64, // Default size
        }
    }

    fn sort_pdas_by_dependencies(&self, pda_inits: &mut Vec<PdaInit>, registry: &AccountRegistry) -> Result<()> {
        let mut dependencies: Vec<(usize, Vec<String>)> = Vec::new();

        // Build dependency list for each PDA
        for (i, pda_init) in pda_inits.iter().enumerate() {
            let account = registry.get_account(&pda_init.account_name)
                .ok_or(SolifyError::DependencyAnalysisFailed(format!("Account not found: {}", pda_init.account_name)))?;
            
            let mut deps = Vec::new();
            for seed in &account.seeds {
                if let SeedType::AccountKey = seed.seed_type {
                    deps.push(seed.value.clone());
                }
            }
            dependencies.push((i, deps));
        }

        // Simple topological sort for PDAs
        let mut sorted_indices = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for i in 0..pda_inits.len() {
            if !visited.contains(&i) {
                self.visit_pda(i, &dependencies, &mut visited, &mut sorted_indices)?;
            }
        }

        // Reorder PDAs based on sorted indices
        let mut sorted_pdas = Vec::new();
        for &idx in &sorted_indices {
            sorted_pdas.push(pda_inits[idx].clone());
        }
        *pda_inits = sorted_pdas;

        Ok(())
    }

    fn visit_pda(
        &self,
        index: usize,
        dependencies: &[(usize, Vec<String>)],
        visited: &mut std::collections::HashSet<usize>,
        sorted: &mut Vec<usize>,
    ) -> Result<()> {
        visited.insert(index);

        let (_, deps) = &dependencies[index];
        for dep_name in deps {
            if let Some(dep_index) = dependencies.iter()
                .position(|(_, d)| d.contains(dep_name))
            {
                if !visited.contains(&dep_index) {
                    self.visit_pda(dep_index, dependencies, visited, sorted)?;
                }
            }
        }

        sorted.push(index);
        Ok(())
    }

    pub fn validate_pda_seeds(&self, pda_init: &PdaInit, registry: &AccountRegistry) -> Result<bool> {
        for seed in &pda_init.seeds {
            match seed.seed_type {
                OutputSeedType::AccountKey => {
                    // Check if the referenced account exists
                    if registry.get_account(&seed.value).is_none() {
                        println!("Warning: PDA seed references unknown account: {}", seed.value);
                        return Ok(false);
                    }
                }
                OutputSeedType::Argument => {
                    // Arguments will be validated during test execution
                    // For now, just log a message
                    println!("PDA uses argument seed: {}", seed.value);
                }
                OutputSeedType::Static => {
                    // Static seeds are always valid
                }
            }
        }
        Ok(true)
    }
}