
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIdl {
    pub name: String,
    pub version: String,
    pub instructions: Vec<Instruction>,
    pub accounts: Vec<AccountDef>,
    pub types: Vec<TypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub name: String,
    pub accounts: Vec<AccountInfo>,
    pub args: Vec<ArgumentDef>,
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub is_optional: bool,
    pub docs: Vec<String>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentDef {
    pub name: String,
    pub arg_type: ArgType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgType {
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
    Vec {
        inner_type: Box<ArgType>,
        max_length: Option<u32>,
    },
    Option { inner_type: Box<ArgType> },
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    Init,
    Mut,
    Seeds(Vec<Seed>),
    HasOne(String),
    Signer,
    Owner(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Seed {
    Static(String),
    AccountKey(String),
    Argument(String),
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub type_kind: TypeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Struct { fields: Vec<FieldDef> },
    Enum { variants: Vec<String> },
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TestMetadata {
    pub instruction_order: Vec<String>,
    pub account_dependencies: Vec<AccountDependency>,
    pub pda_init_sequence: Vec<PdaInit>,
    pub setup_requirements: Vec<SetupRequirement>,
    pub test_cases: Vec<InstructionTestCases>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AccountDependency {
    pub account_name: String,
    pub depends_on: Vec<String>,
    pub is_pda: bool,
    pub is_signer: bool,
    pub is_mut: bool,
    pub must_be_initialized: bool,
    pub initialization_order: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PdaInit {
    pub account_name: String,
    pub seeds: Vec<SeedComponent>,
    pub program_id: Pubkey,
    pub space: Option<u64>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SeedComponent {
    pub seed_type: SeedType,
    pub value: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum SeedType {
    Static,
    AccountKey,
    Argument,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetupRequirement {
    pub requirement_type: SetupType,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum SetupType {
    CreateKeypair,
    FundAccount,
    InitializePda,
    MintTokens,
    CreateAta,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InstructionTestCases {
    pub instruction_name: String,
    pub arguments: Vec<ArgumentInfo>,
    pub positive_cases: Vec<TestCase>,
    pub negative_cases: Vec<TestCase>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ArgumentInfo {
    pub name: String,
    pub arg_type: ArgumentType,
    pub constraints: Vec<ArgumentConstraint>,
    pub is_optional: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
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
    Vec {
        inner_type: Box<ArgumentType>,
        max_length: Option<u32>,
    },
    Option { inner_type: Box<ArgumentType> },
    Struct { name: String },
    Enum {
        name: String,
        variants: Vec<String>,
    },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ArgumentConstraint {
    Min { value: i64 },
    Max { value: i64 },
    Range { min: i64, max: i64 },
    NonZero,
    MaxLength { value: u32 },
    MinLength { value: u32 },
    Custom { description: String },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TestCase {
    pub test_type: TestCaseType,
    pub description: String,
    pub argument_values: Vec<TestArgumentValue>,
    pub expected_outcome: ExpectedOutcome,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TestCaseType {
    Positive,
    NegativeBoundary,
    NegativeType,
    NegativeConstraint,
    NegativeNull,
    NegativeOverflow,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TestArgumentValue {
    pub argument_name: String,
    pub value_type: TestValueType,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TestValueType {
    Valid { description: String },
    Invalid { description: String, reason: String },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ExpectedOutcome {
    Success { state_changes: Vec<String> },
    Failure {
        error_code: Option<String>,
        error_message: String,
    },
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserConfig {
    pub authority: Pubkey,
    pub total_tests_generated: u64,
    pub programs_tested: Vec<ProgramTestHistory>,
    pub created_at: i64,
    pub last_generated_at: i64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ProgramTestHistory {
    pub program_id: Pubkey,
    pub program_name: String,
    pub test_count: u32,
    pub last_generated_at: i64,
    pub idl_hash: [u8; 32],
}