use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;
use solify_common::{TestMetadata, TestCase, IdlData};
use std::fs;
use std::path::Path;

pub mod template;
pub mod writer;

pub use template::*;
pub use writer::*;

/// Generator for TypeScript test files
pub struct TestGenerator<'a> {
    handlebars: Handlebars<'a>,
}

impl<'a> TestGenerator<'a> {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        
        handlebars.register_template_string("main_test", template::MAIN_TEST_TEMPLATE)?;
        
        Ok(Self { handlebars })
    }

    pub fn generate_tests(
        &self,
        idl_data: &IdlData,
        test_metadata: &TestMetadata,
        output_dir: &Path,
        _program_name: &str,
    ) -> Result<GeneratedFiles> {
        let mut generated_files = GeneratedFiles::new();

        // Ensure output directory exists
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;

        // Use IDL name instead of program ID for valid TypeScript identifiers
        // Sanitize the name to ensure it's a valid filename
        let idl_name = Self::sanitize_filename(&idl_data.name);
        
        if idl_name.is_empty() {
            anyhow::bail!("IDL name is empty or contains only invalid characters");
        }

        // Generate single consolidated test file with everything
        let test_content = self.generate_consolidated_test(idl_data, test_metadata, &idl_name)
            .context("Failed to generate test content from template")?;
        
        let test_path = output_dir.join(format!("{}.test.ts", idl_name));
        
        fs::write(&test_path, test_content)
            .with_context(|| format!("Failed to write test file to: {:?}", test_path))?;
        
        generated_files.add_file(test_path);

        Ok(generated_files)
    }

    /// Sanitize filename to remove invalid characters
    pub fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => c,
                _ => '_',
            })
            .collect()
    }

    /// Generate a single consolidated test file with helpers, setup, and all tests
    fn generate_consolidated_test(
        &self,
        idl_data: &IdlData,
        test_metadata: &TestMetadata,
        program_name: &str,
    ) -> Result<String> {
        // Format all test cases in execution order
        let test_suites: Vec<serde_json::Value> = test_metadata.test_cases
            .iter()
            .map(|test_case| {
                // Find the instruction in IDL to get account information
                let idl_instruction = idl_data.instructions.iter()
                    .find(|i| i.name == test_case.instruction_name);
                
                let positive_tests: Vec<serde_json::Value> = test_case.positive_cases
                    .iter()
                    .enumerate()
                    .map(|(i, tc)| self.format_test_case(tc, i + 1, idl_instruction, &test_case.arguments))
                    .collect();

                let negative_tests: Vec<serde_json::Value> = test_case.negative_cases
                    .iter()
                    .enumerate()
                    .map(|(i, tc)| self.format_test_case(tc, i + 1, idl_instruction, &test_case.arguments))
                    .collect();

                // Generate account mappings for this instruction
                let account_mappings = if let Some(instr) = idl_instruction {
                    self.format_account_mappings(&instr.accounts, test_metadata)
                } else {
                    Vec::new()
                };

                json!({
                    "instruction_name": test_case.instruction_name,
                    "positive_tests": positive_tests,
                    "negative_tests": negative_tests,
                    "has_arguments": !test_case.arguments.is_empty(),
                    "arguments": test_case.arguments.iter().map(|arg| {
                        json!({
                            "name": arg.name,
                            "type": format!("{:?}", arg.arg_type),
                            "is_optional": arg.is_optional,
                        })
                    }).collect::<Vec<_>>(),
                    "account_mappings": account_mappings,
                })
            })
            .collect();

        // Format setup steps
        let setup_steps: Vec<serde_json::Value> = test_metadata.setup_requirements
            .iter()
            .map(|req| {
                json!({
                    "type": format!("{:?}", req.requirement_type),
                    "description": req.description,
                    "dependencies": req.dependencies,
                    "is_keypair": matches!(req.requirement_type, solify_common::SetupType::CreateKeypair),
                    "is_fund": matches!(req.requirement_type, solify_common::SetupType::FundAccount),
                })
            })
            .collect();

        // Format PDA initialization sequence
        let pda_init: Vec<serde_json::Value> = test_metadata.pda_init_sequence
            .iter()
            .map(|pda| {
                json!({
                    "account_name": pda.account_name,
                    "program_id": pda.program_id,
                    "seeds": pda.seeds.iter().map(|s| {
                        json!({
                            "type": format!("{:?}", s.seed_type),
                            "value": s.value,
                        })
                    }).collect::<Vec<_>>(),
                })
            })
            .collect();

        // Format account dependencies
        let account_dependencies: Vec<serde_json::Value> = test_metadata.account_dependencies
            .iter()
            .map(|dep| {
                json!({
                    "account_name": dep.account_name,
                    "depends_on": dep.depends_on,
                    "is_pda": dep.is_pda,
                    "must_be_initialized": dep.must_be_initialized,
                })
            })
            .collect();

        let total_positive: usize = test_metadata.test_cases.iter()
            .map(|tc| tc.positive_cases.len())
            .sum();
        let total_negative: usize = test_metadata.test_cases.iter()
            .map(|tc| tc.negative_cases.len())
            .sum();

        let data = json!({
            "program_name": program_name,
            "program_name_upper": program_name.to_uppercase(),
            "program_name_camel": to_camel_case(program_name),
            "instructions": test_metadata.instruction_order,
            "instructions_count": test_metadata.instruction_order.len(),
            "total_tests": total_positive + total_negative,
            "total_positive": total_positive,
            "total_negative": total_negative,
            "pda_count": test_metadata.pda_init_sequence.len(),
            "setup_count": test_metadata.setup_requirements.len(),
            "version": idl_data.version,
            "test_suites": test_suites,
            "setup_steps": setup_steps,
            "pda_init": pda_init,
            "account_dependencies": account_dependencies,
        });

        self.handlebars.render("main_test", &data)
            .context("Failed to render consolidated test template")
    }


    fn format_test_case(
        &self,
        test_case: &TestCase,
        index: usize,
        _idl_instruction: Option<&solify_common::IdlInstruction>,
        argument_infos: &[solify_common::ArgumentInfo],
    ) -> serde_json::Value {
        let expected = match &test_case.expected_outcome {
            solify_common::ExpectedOutcome::Success { state_changes } => {
                json!({
                    "type": "success",
                    "is_success": true,
                    "is_failure": false,
                    "state_changes": state_changes,
                })
            }
            solify_common::ExpectedOutcome::Failure { error_code, error_message } => {
                json!({
                    "type": "failure",
                    "is_success": false,
                    "is_failure": true,
                    "error_code": error_code,
                    "error_message": error_message,
                })
            }
        };

        // Generate actual TypeScript values for arguments
        let arguments: Vec<serde_json::Value> = test_case.argument_values
            .iter()
            .map(|arg_val| {
                // Find the argument info to get the type
                let arg_info = argument_infos.iter()
                    .find(|a| a.name == arg_val.argument_name);
                
                let ts_value = self.generate_typescript_value(
                    &arg_val.value_type,
                    arg_info.map(|a| &a.arg_type),
                );
                
                json!({
                    "name": arg_val.argument_name,
                    "value": ts_value,
                    "is_valid": matches!(arg_val.value_type, solify_common::TestValueType::Valid { .. }),
                })
            })
            .collect();

        json!({
            "index": index,
            "description": test_case.description,
            "test_type": format!("{:?}", test_case.test_type),
            "arguments": arguments,
            "has_arguments": !arguments.is_empty(),
            "expected": expected,
        })
    }

    /// Generate TypeScript value from test value type
    fn generate_typescript_value(
        &self,
        value_type: &solify_common::TestValueType,
        arg_type: Option<&solify_common::ArgumentType>,
    ) -> String {
        match value_type {
            solify_common::TestValueType::Valid { description } => {
                // Parse the description to extract the actual value
                // Descriptions are like: "test_value", "123", "true", etc.
                self.parse_value_from_description(description, arg_type)
            }
            solify_common::TestValueType::Invalid { description, .. } => {
                // For invalid values, we still need to generate code that will fail
                self.parse_value_from_description(description, arg_type)
            }
        }
    }

    /// Parse a value from description string and convert to TypeScript
    fn parse_value_from_description(
        &self,
        description: &str,
        arg_type: Option<&solify_common::ArgumentType>,
    ) -> String {
        // Remove quotes if present
        let cleaned = description.trim_matches('"').trim();
        
        // If description contains quotes, extract the string value
        if cleaned.starts_with('"') && cleaned.ends_with('"') {
            let inner = &cleaned[1..cleaned.len()-1];
            return format!("\"{}\"", inner.replace('\\', "\\\\").replace('"', "\\\""));
        }
        
        // Try to infer type from arg_type if available
        if let Some(typ) = arg_type {
            match typ {
                solify_common::ArgumentType::String { .. } => {
                    // If it's a string type, wrap in quotes
                    format!("\"{}\"", cleaned.replace('\\', "\\\\").replace('"', "\\\""))
                }
                solify_common::ArgumentType::Bool => {
                    // Try to parse as boolean
                    if cleaned.eq_ignore_ascii_case("true") {
                        "true".to_string()
                    } else if cleaned.eq_ignore_ascii_case("false") {
                        "false".to_string()
                    } else {
                        format!("\"{}\"", cleaned)
                    }
                }
                solify_common::ArgumentType::U8
                | solify_common::ArgumentType::U16
                | solify_common::ArgumentType::U32
                | solify_common::ArgumentType::U64
                | solify_common::ArgumentType::U128
                | solify_common::ArgumentType::I8
                | solify_common::ArgumentType::I16
                | solify_common::ArgumentType::I32
                | solify_common::ArgumentType::I64
                | solify_common::ArgumentType::I128 => {
                    // Try to parse as number
                    if cleaned.parse::<i64>().is_ok() || cleaned.parse::<u64>().is_ok() {
                        cleaned.to_string()
                    } else {
                        format!("\"{}\"", cleaned)
                    }
                }
                solify_common::ArgumentType::Pubkey => {
                    // Check if it looks like a pubkey
                    if cleaned.len() == 44 && cleaned.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        format!("new PublicKey(\"{}\")", cleaned)
                    } else {
                        format!("\"{}\"", cleaned)
                    }
                }
                _ => format!("\"{}\"", cleaned),
            }
        } else {
            // Fallback: try to detect type from value
            if cleaned.parse::<i64>().is_ok() || cleaned.parse::<u64>().is_ok() {
                cleaned.to_string()
            } else if cleaned.eq_ignore_ascii_case("true") || cleaned.eq_ignore_ascii_case("false") {
                cleaned.to_lowercase()
            } else {
                format!("\"{}\"", cleaned.replace('\\', "\\\\").replace('"', "\\\""))
            }
        }
    }

    /// Format account mappings for an instruction
    fn format_account_mappings(
        &self,
        accounts: &[solify_common::IdlAccountItem],
        _test_metadata: &TestMetadata,
    ) -> Vec<serde_json::Value> {
        accounts.iter().map(|account| {
            let (account_source, needs_derivation) = if let Some(pda) = &account.pda {
                // Check if PDA depends on instruction arguments
                let has_arg_seed = pda.seeds.iter().any(|seed| seed.kind == "arg");
                
                if has_arg_seed {
                    // PDA depends on arguments - need to derive dynamically
                    let seeds: Vec<String> = pda.seeds.iter().map(|seed| {
                        match seed.kind.as_str() {
                            "arg" => {
                                // Use the argument variable directly - it's already defined above
                                format!("Buffer.from({}, 'utf8')", seed.path)
                            }
                            "account" => {
                                // Get from context accounts
                                format!("testContext.accounts.get(\"{}\")?.publicKey.toBuffer() || Buffer.alloc(32)", seed.path)
                            }
                            "const" => {
                                // Static value
                                format!("Buffer.from(\"{}\")", seed.value)
                            }
                            _ => "Buffer.alloc(32)".to_string(),
                        }
                    }).collect();
                    
                    let seeds_str = seeds.join(",\n      ");
                    (format!("(await PublicKey.findProgramAddress([\n      {}\n    ], program.programId))[0]", seeds_str), true)
                } else {
                    // PDA doesn't depend on arguments - can use from context
                    (format!("testContext.pdas.get(\"{}\")?.[0]", account.name), false)
                }
            } else if account.is_signer {
                // It's a signer - get from context.accounts
                (format!("testContext.accounts.get(\"{}\")?.publicKey", account.name), false)
            } else if account.name == "system_program" {
                ("anchor.web3.SystemProgram.programId".to_string(), false)
            } else {
                // Regular account - try to find in context
                (format!("testContext.accounts.get(\"{}\")?.publicKey", account.name), false)
            };
            
            json!({
                "name": account.name,
                "source": account_source,
                "needs_derivation": needs_derivation,
                "is_signer": account.is_signer,
                "is_writable": account.is_mut,
                "is_pda": account.pda.is_some(),
            })
        }).collect()
    }
}

impl Default for TestGenerator<'_> {
    fn default() -> Self {
        Self::new().expect("Failed to create test generator")
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    
    result
}
