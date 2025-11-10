use std::collections::HashMap;

use anchor_lang::prelude::*;
use crate::types::{IdlData, IdlInstruction, IdlAccountItem};
use crate::error::SolifyError;

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub name: String,
    pub is_pda: bool,
    pub is_signer: bool,
    pub is_mut: bool,
    pub initialized_by: Option<String>,
    pub seeds: Vec<SeedInfo>,
    pub program: Option<String>,
    pub used_in: Vec<String>,
    pub constraints: Vec<ConstraintInfo>,
}

#[derive(Debug, Clone)]
pub struct SeedInfo {
    pub seed_type: SeedType,
    pub value: String,
    pub source: SeedSource,
}

#[derive(Debug, Clone)]
pub enum SeedType {
    Static,
    AccountKey,
    Argument,
}

#[derive(Debug, Clone)]
pub enum SeedSource {
    Authority,
    UserAccount,
    Vault,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ConstraintInfo {
    pub constraint_type: ConstraintType,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConstraintType {
    Init,
    Mut,
    Signer,
    Seeds,
    HasOne,
    Owner,
    Constraint,
    Close,  
    Realloc, 
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: Vec<InstructionNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone)]
pub struct InstructionNode {
    pub name: String,
    pub initializes: Vec<String>,
    pub requires: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
    pub account: String,
}

#[derive(Debug, Clone)]
pub enum DependencyType {
    Initialization,
    SeedDependency,
    Constraint,
}

pub struct AccountRegistry {
    pub accounts: Vec<AccountInfo>,
}

impl AccountRegistry {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }

    pub fn add_or_update_account(&mut self, account: AccountInfo) {
        if let Some(existing) = self.accounts.iter_mut().find(|a| a.name == account.name) {
            // Update existing account
            existing.used_in.extend(account.used_in);
            if account.initialized_by.is_some() {
                existing.initialized_by = account.initialized_by;
            }
            if !account.seeds.is_empty() {
                existing.seeds = account.seeds;
            }
        } else {
            self.accounts.push(account);
        }
    }

    pub fn get_account(&self, name: &str) -> Option<&AccountInfo> {
        self.accounts.iter().find(|a| a.name == name)
    }

    pub fn find_accounts_initialized_by(&self, instruction: &str) -> Vec<&AccountInfo> {
        self.accounts
            .iter()
            .filter(|a| a.initialized_by.as_ref().map_or(false, |i| i == instruction))
            .collect()
    }
}

pub struct DependencyAnalyzerImpl;

impl DependencyAnalyzerImpl {
    pub fn build_account_registry(&self, idl_data: &IdlData, program: &String) -> Result<AccountRegistry> {
        let mut registry = AccountRegistry::new();

        for instruction in &idl_data.instructions {
            self.process_instruction_accounts(instruction, &mut registry, program)?;
        }

        Ok(registry)
    }

    fn process_instruction_accounts(
        &self,
        instruction: &IdlInstruction,
        registry: &mut AccountRegistry,
        program: &String,
    ) -> Result<()> {
        for account_item in &instruction.accounts {
            let account_info = self.parse_account_info(account_item, instruction, program)?;
            registry.add_or_update_account(account_info);
        }
        Ok(())
    }

    fn parse_account_info(
        &self,
        account_item: &IdlAccountItem,
        instruction: &IdlInstruction,
        program: &String,
    ) -> Result<AccountInfo> {
        let mut seeds = Vec::new();
        let mut constraints = Vec::new();
        let mut initialized_by = None;
        let mut is_pda = false;
        let mut program_pda = program.clone();  

        if let Some(pda_info) = &account_item.pda {
            is_pda = true;

            if pda_info.program == Some(program.clone()) {
                program_pda = pda_info.program.clone().unwrap().to_string();
            } else {
                program_pda = program.clone().to_string();
            }
            for idl_seed in &pda_info.seeds {
                let seed_type = match idl_seed.kind.as_str() {
                    "const" | "constant" => SeedType::Static,
                    "arg" | "argument" => SeedType::Argument,
                    "account" => SeedType::AccountKey,
                    _ => {
                        msg!("Unknown seed kind: {}, defaulting to Static", idl_seed.kind);
                        SeedType::Static
                    }
                };
                
                let source = if idl_seed.path.contains("authority") || idl_seed.path.contains("owner") {
                    SeedSource::Authority
                } else if idl_seed.path.contains("user") {
                    SeedSource::UserAccount
                } else if idl_seed.path.contains("vault") {
                    SeedSource::Vault
                } else {
                    SeedSource::Custom(idl_seed.path.clone())
                };
                
                seeds.push(SeedInfo {
                    seed_type,
                    value: idl_seed.path.clone(),
                    source,
                });
            }
            
            constraints.push(ConstraintInfo {
                constraint_type: ConstraintType::Seeds,
                value: Some(format!("{} seeds", seeds.len())),
            });
            
            // msg!("Found PDA account '{}' with {} seeds", account_item.name, seeds.len());
        }

        if account_item.is_mut == true {
            constraints.push(ConstraintInfo {
                constraint_type: ConstraintType::Mut,
                value: Some(String::from("mut")),
            });
        }

        if account_item.is_signer == true {
            constraints.push(ConstraintInfo {
                constraint_type: ConstraintType::Signer,
                value: Some(String::from("signer")),
            });
        }

        let instruction_name_lower = instruction.name.to_lowercase();
        if instruction_name_lower.contains("init") || 
           instruction_name_lower.contains("create") ||
           instruction_name_lower.contains("initialize") {
            initialized_by = Some(instruction.name.clone());
            
            if account_item.is_mut {
                constraints.push(ConstraintInfo {
                    constraint_type: ConstraintType::Init,
                    value: None,
                });
                // msg!("Inferred init constraint for '{}' in instruction '{}'", 
                //      account_item.name, instruction.name);
            }
        }


        Ok(AccountInfo {
            name: account_item.name.clone(),
            is_pda,
            is_signer: account_item.is_signer,
            is_mut: account_item.is_mut,
            initialized_by,
            seeds,
            program: Some(program_pda.clone()),
            used_in: vec![instruction.name.clone()],
            constraints,
        })
    }


    // fn extract_has_one_value(&self, doc: &str) -> Option<String> {
    //     doc.find("has_one = ")
    //         .and_then(|start| {
    //             let rest = &doc[start + 10..];
    //             rest.split_whitespace().next().map(|s| s.trim_matches('"').to_string())
    //         })
    // }

    pub fn build_dependency_graph(
        &self,
        idl_data: &IdlData,
        execution_order: &[String],
        registry: &AccountRegistry,
    ) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        };

        // Create nodes for each instruction in execution order
        for instruction_name in execution_order {
            let instruction = idl_data.instructions
                .iter()
                .find(|i| &i.name == instruction_name)
                .ok_or(SolifyError::InvalidInstructionOrder)?;

            let node = self.create_instruction_node(instruction, registry);
            graph.nodes.push(node);
        }

        // Create edges based on dependencies
        for (i, node) in graph.nodes.iter().enumerate() {
            for account_name in &node.requires {
                if let Some(dep_node_index) = graph.nodes[..i].iter().position(|n| n.initializes.contains(account_name)) {
                    graph.edges.push(DependencyEdge {
                        from: graph.nodes[dep_node_index].name.clone(),
                        to: node.name.clone(),
                        dependency_type: DependencyType::Initialization,
                        account: account_name.clone(),
                    });
                }
            }

            // Add seed dependencies
            for account_name in &node.initializes {
                if let Some(account) = registry.get_account(account_name) {
                    for seed in &account.seeds {
                        if let SeedSource::UserAccount | SeedSource::Vault = seed.source {
                            if let Some(dep_node_index) = graph.nodes[..i].iter().position(|n| n.initializes.contains(&seed.value)) {
                                graph.edges.push(DependencyEdge {
                                    from: graph.nodes[dep_node_index].name.clone(),
                                    to: node.name.clone(),
                                    dependency_type: DependencyType::SeedDependency,
                                    account: account_name.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check for circular dependencies
        self.detect_circular_dependencies(&graph)?;

        Ok(graph)
    }

    fn create_instruction_node(
        &self,
        instruction: &IdlInstruction,
        registry: &AccountRegistry,
    ) -> InstructionNode {
        let mut initializes = Vec::new();
        let mut requires = Vec::new();

        for account_item in &instruction.accounts {
            if let Some(account) = registry.get_account(&account_item.name) {
                if account.initialized_by.as_ref() == Some(&instruction.name) {
                    initializes.push(account.name.clone());
                } else {
                    requires.push(account.name.clone());
                }
            }
        }

        InstructionNode {
            name: instruction.name.clone(),
            initializes,
            requires,
        }
    }

    fn detect_circular_dependencies(&self, graph: &DependencyGraph) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for node in &graph.nodes {
            if !visited.contains(&node.name) {
                if self.has_cycle(
                    graph,
                    &node.name,
                    &mut visited,
                    &mut recursion_stack,
                )? {
                    return Err(SolifyError::CircularDependency.into());
                }
            }
        }

        Ok(())
    }

    fn has_cycle(
        &self,
        graph: &DependencyGraph,
        node_name: &str,
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
    ) -> Result<bool> {
        visited.insert(node_name.to_string());
        recursion_stack.insert(node_name.to_string());

        for edge in &graph.edges {
            if edge.from == node_name {
                if recursion_stack.contains(&edge.to) {
                    return Ok(true);
                }
                if !visited.contains(&edge.to) {
                    if self.has_cycle(graph, &edge.to, visited, recursion_stack)? {
                        return Ok(true);
                    }
                }
            }
        }

        recursion_stack.remove(node_name);
        Ok(false)
    }

    // kahn's algorithm
    pub fn topological_sort(&self, graph: &DependencyGraph) -> Result<Vec<String>> {
        let mut in_degree = HashMap::with_capacity(graph.nodes.len());
        
        // Initialize in-degree for all nodes
        for node in &graph.nodes {
            in_degree.insert(node.name.clone(), 0);
        }

        // Calculate in-degree
        for edge in &graph.edges {
            *in_degree.get_mut(&edge.to).unwrap() += 1;
        }

        // Find nodes with in-degree 0
        let mut queue: std::collections::VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted = Vec::new();
        let mut visited_edges = 0;

        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());

            // Decrease in-degree of neighbors
            for edge in &graph.edges {
                if edge.from == node {
                    let to_degree = in_degree.get_mut(&edge.to).unwrap();
                    *to_degree -= 1;
                    visited_edges += 1;

                    if *to_degree == 0 {
                        queue.push_back(edge.to.clone());
                    }
                }
            }
        }

        // Check if we have a cycle
        if visited_edges != graph.edges.len() {
            return Err(SolifyError::CircularDependency.into());
        }

        Ok(sorted)
    }
}