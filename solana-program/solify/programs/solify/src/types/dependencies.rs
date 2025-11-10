use anchor_lang::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct AccountDependency {
    #[max_len(10)]
    pub account_name: String,
    #[max_len(5, 15)]
    pub depends_on: Vec<String>,
    pub is_pda: bool,
    pub is_signer: bool,
    pub is_mut: bool,
    pub must_be_initialized: bool,
    pub initialization_order: u8,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct PdaInit {
    #[max_len(10)]
    pub account_name: String,
    #[max_len(10)]
    pub seeds: Vec<SeedComponent>,
    pub program_id: Pubkey,
    pub space: Option<u64>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct SeedComponent {
    pub seed_type: SeedType,
    #[max_len(10)]
    pub value: String,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum SeedType {
    Static,
    AccountKey,
    Argument,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct SetupRequirement {
    pub requirement_type: SetupType,
    #[max_len(20)]
    pub description: String,
    #[max_len(5, 15)]
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum SetupType {
    CreateKeypair,
    FundAccount,
    InitializePda,
    MintTokens,
    CreateAta,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct InstructionTestCases {
    #[max_len(10)]
    pub instruction_name: String,
    #[max_len(3)]
    pub arguments: Vec<ArgumentInfo>,
    #[max_len(3)]
    pub positive_cases: Vec<TestCase>,
    #[max_len(3)]
    pub negative_cases: Vec<TestCase>,
}

// // Simplified version for events
// #[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
// pub struct InstructionTestCasesEvent {
//     #[max_len(50)]
//     pub instruction_name: String,
//     #[max_len(20)]
//     pub arguments: Vec<ArgumentInfoEvent>,
//     pub positive_case_count: u32,
//     pub negative_case_count: u32,
// }

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct ArgumentInfo {
    #[max_len(10)]
    pub name: String,
    pub arg_type: ArgumentType,
    #[max_len(5)]
    pub constraints: Vec<ArgumentConstraint>,
    pub is_optional: bool,
}

// // Simplified version for events
// #[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
// pub struct ArgumentInfoEvent {
//     #[max_len(50)]
//     pub name: String,
//     #[max_len(100)]
//     pub arg_type_name: String,
//     pub is_optional: bool,
// }

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum ArgumentType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Bool,
    String { max_length: Option<u32> },
    Pubkey,
    VecType { #[max_len(10)] inner_type_name: String, max_length: Option<u32> },
    OptionType { #[max_len(10)] inner_type_name: String },
}

impl ArgumentType {
    pub fn to_string(&self) -> String {
        match self {
            ArgumentType::U8 => "u8".to_string(),
            ArgumentType::U16 => "u16".to_string(),
            ArgumentType::U32 => "u32".to_string(),
            ArgumentType::U64 => "u64".to_string(),
            ArgumentType::U128 => "u128".to_string(),
            ArgumentType::I8 => "i8".to_string(),
            ArgumentType::I16 => "i16".to_string(),
            ArgumentType::I32 => "i32".to_string(),
            ArgumentType::I64 => "i64".to_string(),
            ArgumentType::I128 => "i128".to_string(),
            ArgumentType::Bool => "bool".to_string(),
            ArgumentType::String { max_length } => {
                if let Some(max) = max_length {
                    format!("String(max:{})", max)
                } else {
                    "String".to_string()
                }
            },
            ArgumentType::Pubkey => "Pubkey".to_string(),
            ArgumentType::VecType { inner_type_name, max_length } => {
                if let Some(max) = max_length {
                    format!("Vec<{}>(max:{})", inner_type_name, max)
                } else {
                    format!("Vec<{}>", inner_type_name)
                }
            },
            ArgumentType::OptionType { inner_type_name } => {
                format!("Option<{}>", inner_type_name)
            },
        }
    }
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum ArgumentConstraint {
    Min { value: i64 },
    Max { value: i64 },
    Range { min: i64, max: i64 },
    NonZero,
    MaxLength { value: u32 },
    MinLength { value: u32 },
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct TestCase {
    pub test_type: TestCaseType,
    #[max_len(10)]
    pub description: String,
    #[max_len(3)]
    pub argument_values: Vec<TestArgumentValue>,
    pub expected_outcome: ExpectedOutcome,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum TestCaseType {
    Positive,
    NegativeBoundary,
    NegativeType,
    NegativeConstraint,
    NegativeNull,
    NegativeOverflow,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub struct TestArgumentValue {
    #[max_len(10)]
    pub argument_name: String,
    pub value_type: TestValueType,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum TestValueType {
    Valid { #[max_len(20)] description: String },
    Invalid { #[max_len(20)] description: String, #[max_len(20)] reason: String },
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, Serialize, Deserialize, InitSpace)]
pub enum ExpectedOutcome {
    Success { #[max_len(5, 15)] state_changes: Vec<String> },
    Failure { #[max_len(5)] error_code: Option<String>, #[max_len(20)] error_message: String },
}