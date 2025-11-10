use anchor_lang::prelude::*;
use crate::types::{
    IdlData,
    IdlInstruction,
    IdlField,
    InstructionTestCases,
    TestCase,
    TestCaseType,
    TestArgumentValue,
    TestValueType,
    ExpectedOutcome,
    ArgumentInfo,
    ArgumentType,
    ArgumentConstraint,
};
use crate::error::SolifyError;

pub struct TestCaseGenerator;

impl TestCaseGenerator {
    pub fn generate_test_cases(
        &self,
        idl_data: &IdlData,
        execution_order: &[String]
    ) -> Result<Vec<InstructionTestCases>> {
        let mut all_test_cases = Vec::new();

        for instruction_name in execution_order {
            let instruction = idl_data.instructions
                .iter()
                .find(|i| &i.name == instruction_name)
                .ok_or(SolifyError::InvalidInstructionOrder)?;

            let test_cases = self.generate_instruction_test_cases(instruction)?;
            all_test_cases.push(test_cases);
        }

        Ok(all_test_cases)
    }

    fn generate_instruction_test_cases(
        &self,
        instruction: &IdlInstruction
    ) -> Result<InstructionTestCases> {
        let arguments = self.parse_arguments(&instruction.args)?;
        let positive_cases = self.generate_positive_cases(&instruction.name, &arguments)?;
        let negative_cases = self.generate_negative_cases(&instruction.name, &arguments)?;

        Ok(InstructionTestCases {
            instruction_name: instruction.name.clone(),
            arguments,
            positive_cases,
            negative_cases,
        })
    }

    fn parse_arguments(&self, args: &[IdlField]) -> Result<Vec<ArgumentInfo>> {
        let mut argument_infos = Vec::new();

        for arg in args {
            let arg_type = self.parse_argument_type(&arg)?;
            let constraints = self.extract_constraints_from_docs(&arg)?;

            argument_infos.push(ArgumentInfo {
                name: arg.name.clone(),
                arg_type,
                constraints,
                is_optional: false, // Would need to parse from IDL
            });
        }

        Ok(argument_infos)
    }

    fn parse_argument_type(&self, field_type: &IdlField) -> Result<ArgumentType> {
        match field_type.field_type.as_str() {
            "u8" => Ok(ArgumentType::U8),
            "u16" => Ok(ArgumentType::U16),
            "u32" => Ok(ArgumentType::U32),
            "u64" => Ok(ArgumentType::U64),
            "u128" => Ok(ArgumentType::U128),
            "i8" => Ok(ArgumentType::I8),
            "i16" => Ok(ArgumentType::I16),
            "i32" => Ok(ArgumentType::I32),
            "i64" => Ok(ArgumentType::I64),
            "i128" => Ok(ArgumentType::I128),
            "bool" => Ok(ArgumentType::Bool),
            "string" => Ok(ArgumentType::String { max_length: None }),
            "publicKey" => Ok(ArgumentType::Pubkey),
            _ => Ok(ArgumentType::VecType { inner_type_name: "u8".to_string(), max_length: None }),
        }
    }

    fn extract_constraints_from_docs(&self, field_type: &IdlField) -> Result<Vec<ArgumentConstraint>> {
    let mut constraints = Vec::new();

    // This would typically parse constraints from field docs
    // For now, we'll add some basic constraints based on type
    match field_type.field_type.as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" => {
            constraints.push(ArgumentConstraint::Min { value: 0 });
            constraints.push(ArgumentConstraint::NonZero);
        }
        "string" => {
            constraints.push(ArgumentConstraint::MinLength { value: 1 });
            constraints.push(ArgumentConstraint::MaxLength { value: 100 });
        }
        _ => {}
    }

    Ok(constraints)
}

    fn generate_positive_cases(
        &self,
    instruction_name: &str,
    arguments: &[ArgumentInfo]
) -> Result<Vec<TestCase>> {
    let mut positive_cases = Vec::new();

    // Generate basic positive case
    let basic_case = self.create_basic_positive_case(instruction_name, arguments)?;
    positive_cases.push(basic_case);

    // Generate boundary cases for numeric types
    for arg in arguments {
        if let Some(boundary_cases) = self.generate_boundary_cases(arg)? {
            positive_cases.extend(boundary_cases);
        }
    }

    Ok(positive_cases)
}

fn create_basic_positive_case(
    &self,
    instruction_name: &str,
    arguments: &[ArgumentInfo]
) -> Result<TestCase> {
    let argument_values = arguments
        .iter()
        .map(|arg| {
            let value = match &arg.arg_type {
                | ArgumentType::U8
                | ArgumentType::U16
                | ArgumentType::U32
                | ArgumentType::U64
                | ArgumentType::U128 => "1000".to_string(),
                | ArgumentType::I8
                | ArgumentType::I16
                | ArgumentType::I32
                | ArgumentType::I64
                | ArgumentType::I128 => "500".to_string(),
                ArgumentType::Bool => "true".to_string(),
                ArgumentType::String { .. } => "\"test_value\"".to_string(),
                ArgumentType::Pubkey => "authority.publicKey".to_string(),
                _ => "/* valid value */".to_string(),
            };

            TestArgumentValue {
                argument_name: arg.name.clone(),
                value_type: TestValueType::Valid {
                    description: value,
                },
            }
        })
        .collect();

    Ok(TestCase {
        test_type: TestCaseType::Positive,
        description: format!("{} - valid inputs", instruction_name),
        argument_values,
        expected_outcome: ExpectedOutcome::Success {
            state_changes: vec![
                "Account state updated successfully".to_string(),
                "Instruction executed without errors".to_string()
            ],
        },
    })
}

fn generate_boundary_cases(&self, argument: &ArgumentInfo) -> Result<Option<Vec<TestCase>>> {
    let mut boundary_cases = Vec::new();

    for constraint in &argument.constraints {
        match constraint {
            ArgumentConstraint::Min { value } => {
                boundary_cases.push(TestCase {
                    test_type: TestCaseType::Positive,
                    description: format!("{} - minimum value", argument.name),
                    argument_values: vec![TestArgumentValue {
                        argument_name: argument.name.clone(),
                        value_type: TestValueType::Valid {
                            description: value.to_string(),
                        },
                    }],
                    expected_outcome: ExpectedOutcome::Success {
                        state_changes: vec!["Minimum value accepted".to_string()],
                    },
                });
            }
            ArgumentConstraint::Max { value } => {
                boundary_cases.push(TestCase {
                    test_type: TestCaseType::Positive,
                    description: format!("{} - maximum value", argument.name),
                    argument_values: vec![TestArgumentValue {
                        argument_name: argument.name.clone(),
                        value_type: TestValueType::Valid {
                            description: value.to_string(),
                        },
                    }],
                    expected_outcome: ExpectedOutcome::Success {
                        state_changes: vec!["Maximum value accepted".to_string()],
                    },
                });
            }
            _ => {}
        }
    }

    if boundary_cases.is_empty() {
        Ok(None)
    } else {
        Ok(Some(boundary_cases))
    }
}

    fn generate_negative_cases(
        &self,
    instruction_name: &str,
    arguments: &[ArgumentInfo]
) -> Result<Vec<TestCase>> {
    let mut negative_cases = Vec::new();

    for arg in arguments {
        negative_cases.extend(self.generate_argument_negative_cases(instruction_name, arg)?);
    }

    // Add combined negative case
    if arguments.len() > 1 {
        negative_cases.push(self.create_combined_negative_case(instruction_name, arguments)?);
    }

    Ok(negative_cases)
}

fn generate_argument_negative_cases(
    &self,
    instruction_name: &str,
    argument: &ArgumentInfo
) -> Result<Vec<TestCase>> {
    let mut negative_cases = Vec::new();

    // Generate constraint violation cases
    for constraint in &argument.constraints {
        if
            let Some(test_case) = self.create_constraint_violation_case(
                instruction_name,
                argument,
                constraint
            )?
        {
            negative_cases.push(test_case);
        }
    }

    // Generate type-specific negative cases
    match &argument.arg_type {
        | ArgumentType::U8
        | ArgumentType::U16
        | ArgumentType::U32
        | ArgumentType::U64
        | ArgumentType::U128 => {
            negative_cases.extend(
                self.generate_numeric_negative_cases(instruction_name, argument)?
            );
        }
        ArgumentType::String { .. } => {
            negative_cases.extend(self.generate_string_negative_cases(instruction_name, argument)?);
        }
        ArgumentType::Pubkey => {
            negative_cases.extend(self.generate_pubkey_negative_cases(instruction_name, argument)?);
        }
        _ => {}
    }

    Ok(negative_cases)
}

fn create_constraint_violation_case(
    &self,
    instruction_name: &str,
    argument: &ArgumentInfo,
    constraint: &ArgumentConstraint
) -> Result<Option<TestCase>> {
    let test_case = match constraint {
        ArgumentConstraint::Min { value } =>
            Some(TestCase {
                test_type: TestCaseType::NegativeBoundary,
                description: format!("{} - {} below minimum", instruction_name, argument.name),
                argument_values: vec![TestArgumentValue {
                    argument_name: argument.name.clone(),
                    value_type: TestValueType::Invalid {
                        description: (value - 1).to_string(),
                        reason: format!("Below minimum value of {}", value),
                    },
                }],
                expected_outcome: ExpectedOutcome::Failure {
                    error_code: Some("ConstraintViolation".to_string()),
                    error_message: format!("{} must be at least {}", argument.name, value),
                },
            }),
        ArgumentConstraint::Max { value } =>
            Some(TestCase {
                test_type: TestCaseType::NegativeBoundary,
                description: format!("{} - {} above maximum", instruction_name, argument.name),
                argument_values: vec![TestArgumentValue {
                    argument_name: argument.name.clone(),
                    value_type: TestValueType::Invalid {
                        description: (value + 1).to_string(),
                        reason: format!("Above maximum value of {}", value),
                    },
                }],
                expected_outcome: ExpectedOutcome::Failure {
                    error_code: Some("ConstraintViolation".to_string()),
                    error_message: format!("{} must be at most {}", argument.name, value),
                },
            }),
        ArgumentConstraint::NonZero =>
            Some(TestCase {
                test_type: TestCaseType::NegativeConstraint,
                description: format!("{} - {} is zero", instruction_name, argument.name),
                argument_values: vec![TestArgumentValue {
                    argument_name: argument.name.clone(),
                    value_type: TestValueType::Invalid {
                        description: "0".to_string(),
                        reason: "Must be non-zero".to_string(),
                    },
                }],
                expected_outcome: ExpectedOutcome::Failure {
                    error_code: Some("ZeroAmount".to_string()),
                    error_message: format!("{} cannot be zero", argument.name),
                },
            }),
        _ => None,
    };

    Ok(test_case)
}

fn generate_numeric_negative_cases(
    &self,
    instruction_name: &str,
    argument: &ArgumentInfo
) -> Result<Vec<TestCase>> {
    let mut cases = Vec::new();

    // Overflow case
    cases.push(TestCase {
        test_type: TestCaseType::NegativeOverflow,
        description: format!("{} - {} overflow", instruction_name, argument.name),
        argument_values: vec![TestArgumentValue {
            argument_name: argument.name.clone(),
            value_type: TestValueType::Invalid {
                description: "u64::MAX".to_string(),
                reason: "Potential arithmetic overflow".to_string(),
            },
        }],
        expected_outcome: ExpectedOutcome::Failure {
            error_code: Some("Overflow".to_string()),
            error_message: "Arithmetic overflow".to_string(),
        },
    });

    // Negative value for unsigned type
    cases.push(TestCase {
        test_type: TestCaseType::NegativeType,
        description: format!("{} - {} negative value", instruction_name, argument.name),
        argument_values: vec![TestArgumentValue {
            argument_name: argument.name.clone(),
            value_type: TestValueType::Invalid {
                description: "-1".to_string(),
                reason: "Unsigned type cannot be negative".to_string(),
            },
        }],
        expected_outcome: ExpectedOutcome::Failure {
            error_code: Some("InvalidType".to_string()),
            error_message: "Unsigned integer cannot be negative".to_string(),
        },
    });

    Ok(cases)
}

fn generate_string_negative_cases(
    &self,
    instruction_name: &str,
    argument: &ArgumentInfo
) -> Result<Vec<TestCase>> {
    let mut cases = Vec::new();

    // Empty string
    cases.push(TestCase {
        test_type: TestCaseType::NegativeNull,
        description: format!("{} - {} empty string", instruction_name, argument.name),
        argument_values: vec![TestArgumentValue {
            argument_name: argument.name.clone(),
            value_type: TestValueType::Invalid {
                description: "\"\"".to_string(),
                reason: "String cannot be empty".to_string(),
            },
        }],
        expected_outcome: ExpectedOutcome::Failure {
            error_code: Some("EmptyString".to_string()),
            error_message: "String cannot be empty".to_string(),
        },
    });

    // Too long string
    cases.push(TestCase {
        test_type: TestCaseType::NegativeBoundary,
        description: format!("{} - {} too long", instruction_name, argument.name),
        argument_values: vec![TestArgumentValue {
            argument_name: argument.name.clone(),
            value_type: TestValueType::Invalid {
                description: "\"a\".repeat(1000)".to_string(),
                reason: "Exceeds maximum length".to_string(),
            },
        }],
        expected_outcome: ExpectedOutcome::Failure {
            error_code: Some("StringTooLong".to_string()),
            error_message: "String exceeds maximum length".to_string(),
        },
    });

    Ok(cases)
}

fn generate_pubkey_negative_cases(
    &self,
    instruction_name: &str,
    argument: &ArgumentInfo
) -> Result<Vec<TestCase>> {
    let mut cases = Vec::new();

    // Invalid pubkey
    cases.push(TestCase {
        test_type: TestCaseType::NegativeType,
        description: format!("{} - {} invalid pubkey", instruction_name, argument.name),
        argument_values: vec![TestArgumentValue {
            argument_name: argument.name.clone(),
            value_type: TestValueType::Invalid {
                description: "Keypair.generate().publicKey".to_string(),
                reason: "Account not initialized".to_string(),
            },
        }],
        expected_outcome: ExpectedOutcome::Failure {
            error_code: Some("AccountNotInitialized".to_string()),
            error_message: "Account has not been initialized".to_string(),
        },
    });

    Ok(cases)
}

    fn create_combined_negative_case(
        &self,
    instruction_name: &str,
    arguments: &[ArgumentInfo]
) -> Result<TestCase> {
    let argument_values = arguments
        .iter()
        .map(|arg| TestArgumentValue {
            argument_name: arg.name.clone(),
            value_type: TestValueType::Invalid {
                description: "invalid".to_string(),
                reason: "Multiple validation failures".to_string(),
            },
        })
        .collect();

    Ok(TestCase {
        test_type: TestCaseType::NegativeConstraint,
        description: format!("{} - all arguments invalid", instruction_name),
        argument_values,
        expected_outcome: ExpectedOutcome::Failure {
            error_code: None,
            error_message: "Multiple validation errors".to_string(),
        },
    })
}
}