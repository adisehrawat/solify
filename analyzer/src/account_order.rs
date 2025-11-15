use solify_common::types::{AccountDependency};
use solify_common::errors::{SolifyError, Result};
use crate::dependency_analyzer::*;
pub struct AccountOrder;

impl AccountOrder {
    pub fn generate_account_dependencies(
        &self,
        graph: &DependencyGraph,
        registry: &AccountRegistry,
    ) -> Result<Vec<AccountDependency>> {
        let mut account_dependencies = Vec::new();
        let mut initialization_order = 0u8;

        // Get topological order of instructions
        let sorted_instructions = self.get_sorted_instructions(graph)?;

        // Create account dependencies based on instruction order
        for instruction_name in sorted_instructions {
            if let Some(instruction_node) = graph.nodes.iter().find(|n| n.name == instruction_name) {
                for account_name in &instruction_node.initializes {
                    if let Some(account) = registry.get_account(account_name) {
                        let depends_on = self.get_account_dependencies(account, registry);
                        
                        account_dependencies.push(AccountDependency {
                            account_name: account_name.clone(),
                            depends_on,
                            is_pda: account.is_pda,
                            is_signer: account.is_signer,
                            is_mut: account.is_mut,
                            must_be_initialized: account.initialized_by.is_some(),
                            initialization_order,
                        });

                        initialization_order = initialization_order.saturating_add(1);
                    }
                }
            }
        }

        // Add external accounts (not initialized by any instruction)
        for account in &registry.accounts {
            if account.initialized_by.is_none() && !account_dependencies.iter().any(|ad| ad.account_name == account.name) {
                account_dependencies.push(AccountDependency {
                    account_name: account.name.clone(),
                    depends_on: Vec::new(),
                    is_pda: account.is_pda,
                    is_signer: account.is_signer,
                    is_mut: account.is_mut,
                    must_be_initialized: false,
                    initialization_order,
                });
                initialization_order = initialization_order.saturating_add(1);
            }
        }

        Ok(account_dependencies)
    }

    fn get_sorted_instructions(&self, graph: &DependencyGraph) -> Result<Vec<String>> {
        // Simple topological sort implementation
        let mut in_degree = std::collections::HashMap::new();
        
        for node in &graph.nodes {
            in_degree.insert(node.name.clone(), 0);
        }

        for edge in &graph.edges {
            *in_degree.get_mut(&edge.to).unwrap() += 1;
        }

        let mut queue: std::collections::VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());

            for edge in &graph.edges {
                if edge.from == node {
                    let degree = in_degree.get_mut(&edge.to).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(edge.to.clone());
                    }
                }
            }
        }

        if sorted.len() != graph.nodes.len() {
            return Err(SolifyError::CircularDependency.into());
        }

        Ok(sorted)
    }

    fn get_account_dependencies(
        &self,
        account: &AccountInfo,
        _registry: &AccountRegistry,
    ) -> Vec<String> {
        let mut dependencies = Vec::new();

        for seed in &account.seeds {
            if let SeedType::AccountKey = seed.seed_type {
                dependencies.push(seed.value.clone());
            }
        }

        // Add constraint-based dependencies
        for constraint in &account.constraints {
            if let ConstraintType::HasOne = constraint.constraint_type {
                if let Some(value) = &constraint.value {
                    dependencies.push(value.clone());
                }
            }
        }

        dependencies
    }

    pub fn validate_account_flow(&self, dependencies: &[AccountDependency]) -> Result<bool> {
        // Check for circular dependencies in account initialization
        let mut graph = std::collections::HashMap::new();
        
        for dep in dependencies {
            graph.insert(dep.account_name.clone(), dep.depends_on.clone());
        }

        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for account in graph.keys() {
            if !visited.contains(account) {
                if self.has_circular_dependency(account, &graph, &mut visited, &mut recursion_stack) {
                    return Err(SolifyError::CircularDependency.into());
                }
            }
        }

        Ok(true)
    }

    fn has_circular_dependency(
        &self,
        account: &str,
        graph: &std::collections::HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(account.to_string());
        recursion_stack.insert(account.to_string());

        if let Some(dependencies) = graph.get(account) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if self.has_circular_dependency(dep, graph, visited, recursion_stack) {
                        return true;
                    }
                } else if recursion_stack.contains(dep) {
                    return true;
                }
            }
        }

        recursion_stack.remove(account);
        false
    }
}