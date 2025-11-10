use anchor_lang::prelude::*;

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct AccountDependency {
    pub account_name: String,
    pub depends_on: Vec<String>,
    pub is_pda: bool,
    pub is_signer: bool,
    pub is_mut: bool,
    pub must_be_initialized: bool,
    pub initialization_order: u8,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct PdaInit {
    pub account_name: String,
    pub seeds: Vec<SeedComponent>,
    pub program_id: Pubkey,
    pub space: Option<u64>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct SeedComponent {
    pub seed_type: SeedType,
    pub value: String,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum SeedType {
    Static,
    AccountKey,
    Argument,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct SetupRequirement {
    pub requirement_type: SetupType,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum SetupType {
    CreateKeypair,
    FundAccount,
    InitializePda,
    MintTokens,
    CreateAta,
}

#[derive(Clone, Debug)]
pub struct InstructionTestCases {
    pub instruction_name: String,
    pub arguments: Vec<ArgumentInfo>,
    pub positive_cases: Vec<TestCase>,
    pub negative_cases: Vec<TestCase>,
}

// Simplified version for events
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct InstructionTestCasesEvent {
    pub instruction_name: String,
    pub arguments: Vec<ArgumentInfoEvent>,
    pub positive_case_count: u32,
    pub negative_case_count: u32,
}

#[derive(Clone, Debug)]
pub struct ArgumentInfo {
    pub name: String,
    pub arg_type: ArgumentType,
    pub constraints: Vec<ArgumentConstraint>,
    pub is_optional: bool,
}

// Simplified version for events
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct ArgumentInfoEvent {
    pub name: String,
    pub arg_type_name: String,
    pub is_optional: bool,
}

#[derive(Clone, Debug)]
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
    Vec { inner_type: Box<ArgumentType>, max_length: Option<u32> },
    Option { inner_type: Box<ArgumentType> },
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
            ArgumentType::Vec { inner_type, max_length } => {
                if let Some(max) = max_length {
                    format!("Vec<{}>(max:{})", inner_type.to_string(), max)
                } else {
                    format!("Vec<{}>", inner_type.to_string())
                }
            },
            ArgumentType::Option { inner_type } => {
                format!("Option<{}>", inner_type.to_string())
            },
        }
    }
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum ArgumentConstraint {
    Min { value: i64 },
    Max { value: i64 },
    Range { min: i64, max: i64 },
    NonZero,
    MaxLength { value: u32 },
    MinLength { value: u32 },
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct TestCase {
    pub test_type: TestCaseType,
    pub description: String,
    pub argument_values: Vec<TestArgumentValue>,
    pub expected_outcome: ExpectedOutcome,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum TestCaseType {
    Positive,
    NegativeBoundary,
    NegativeType,
    NegativeConstraint,
    NegativeNull,
    NegativeOverflow,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct TestArgumentValue {
    pub argument_name: String,
    pub value_type: TestValueType,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum TestValueType {
    Valid { description: String },
    Invalid { description: String, reason: String },
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub enum ExpectedOutcome {
    Success { state_changes: Vec<String> },
    Failure { error_code: Option<String>, error_message: String },
}