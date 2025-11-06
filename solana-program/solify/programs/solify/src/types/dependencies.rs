use anchor_lang::prelude::{borsh::{BorshDeserialize, BorshSerialize}, *};

#[derive(Clone, Debug)]
pub struct AccountDependency {
    pub account_name: String,
    pub depends_on: Vec<String>,
    pub is_pda: bool,
    pub is_signer: bool,
    pub is_mut: bool,
    pub must_be_initialized: bool,
    pub initialization_order: u8,
}

#[derive(Clone, Debug)]
pub struct PdaInit {
    pub account_name: String,
    pub seeds: Vec<SeedComponent>,
    pub program_id: Pubkey,
    pub space: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct SeedComponent {
    pub seed_type: SeedType,
    pub value: String,
}

#[derive(Clone, Debug)]
pub enum SeedType {
    Static,
    AccountKey,
    Argument,
}

#[derive(Clone, Debug)]
pub struct SetupRequirement {
    pub requirement_type: SetupType,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct ArgumentInfo {
    pub name: String,
    pub arg_type: ArgumentType,
    pub constraints: Vec<ArgumentConstraint>,
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

#[derive(Clone, Debug)]
pub enum ArgumentConstraint {
    Min { value: i64 },
    Max { value: i64 },
    Range { min: i64, max: i64 },
    NonZero,
    MaxLength { value: u32 },
    MinLength { value: u32 },
}

#[derive(Clone, Debug)]
pub struct TestCase {
    pub test_type: TestCaseType,
    pub description: String,
    pub argument_values: Vec<TestArgumentValue>,
    pub expected_outcome: ExpectedOutcome,
}

#[derive(Clone, Debug)]
pub enum TestCaseType {
    Positive,
    NegativeBoundary,
    NegativeType,
    NegativeConstraint,
    NegativeNull,
    NegativeOverflow,
}

#[derive(Clone, Debug)]
pub struct TestArgumentValue {
    pub argument_name: String,
    pub value_type: TestValueType,
}

#[derive(Clone, Debug)]
pub enum TestValueType {
    Valid { description: String },
    Invalid { description: String, reason: String },
}

#[derive(Clone, Debug)]
pub enum ExpectedOutcome {
    Success { state_changes: Vec<String> },
    Failure { error_code: Option<String>, error_message: String },
}