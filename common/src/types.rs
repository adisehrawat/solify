
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;


#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlData {
    pub name: String,
    pub version: String,
    pub instructions: Vec<IdlInstruction>,
    #[serde(default)]
    pub accounts: Vec<IdlAccount>,
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(default)]
    pub errors: Vec<IdlError>,
    #[serde(default)]
    pub constants: Vec<IdlConstant>,
    #[serde(default)]
    pub events: Vec<IdlEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlError {
    pub code: u32,
    pub name: String,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlConstant {
    pub name: String,
    pub constant_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlEvent {
    pub name: String,
    #[serde(default)]
    pub discriminator: Vec<u8>,
    pub fields: Vec<IdlField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlInstruction {
    pub name: String,
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlAccountItem {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub is_optional: bool,
    pub docs: Vec<String>,
    pub pda: Option<IdlPda>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlPda {
    pub seeds: Vec<IdlSeed>,
    #[serde(default)]
    pub program: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlSeed {
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlAccount {
    pub name: String,
    pub fields: Vec<IdlField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlField {
    pub name: String,
    pub field_type: String, 
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct IdlTypeDef {
    pub name: String,
    pub kind: String, 
    pub fields: Vec<String>, 
}



#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ParsedIdl {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub metadata: IdlMetadata,
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub accounts: Vec<AccountDef>,
    #[serde(default)]
    pub types: Vec<TypeDef>,
    #[serde(default)]
    pub errors: Vec<ErrorDef>,
    #[serde(default)]
    pub constants: Vec<ConstantDef>,
    #[serde(default)]
    pub events: Vec<EventDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ErrorDef {
    pub code: u32,
    pub name: String,
    #[serde(default)]
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ConstantDef {
    pub name: String,
    #[serde(rename = "type")]
    pub constant_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EventDef {
    pub name: String,
    #[serde(default)]
    pub discriminator: Vec<u8>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Default)]
pub struct IdlMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub spec: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Instruction {
    pub name: String,
    #[serde(default)]
    pub discriminator: Vec<u8>,
    pub accounts: Vec<AccountInfo>,
    pub args: Vec<ArgumentDef>,
    #[serde(default)]
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AccountInfo {
    pub name: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub signer: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub pda: Option<PdaConfig>,
    #[serde(default)]
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PdaConfig {
    pub seeds: Vec<PdaSeed>,
    #[serde(default)]
    pub program: Option<PdaProgram>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PdaProgram {
    pub kind: String,
    #[serde(default)]
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PdaSeed {
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ArgumentDef {
    pub name: String,
    
    #[serde(rename = "type")]
    pub arg_type: IdlType,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(untagged)]
pub enum IdlType {
    Simple(String),
    Vec {
        vec: Box<IdlType>
    },
    Option {
        option: Box<IdlType>
    },
    Array {
        array: (Box<IdlType>, usize)
    },
    Defined {
        defined: DefinedType
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(untagged)]
pub enum DefinedType {
    Simple(String),
    Generic {
        name: String,
        #[serde(default)]
        generics: Vec<IdlType>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AccountDef {
    pub name: String,
    
    #[serde(default)]
    pub discriminator: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FieldDef {
    pub name: String,
    
    #[serde(rename = "type")]
    pub field_type: IdlType,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TypeDef {
    pub name: String,
    
    #[serde(rename = "type")]
    pub type_kind: TypeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(tag = "kind")]
pub enum TypeKind {
    #[serde(rename = "struct")]
    Struct { 
        fields: Vec<FieldDef> 
    },
    
    #[serde(rename = "enum")]
    Enum { 
        variants: Vec<EnumVariant> 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EnumVariant {
    pub name: String,
    
    #[serde(default)]
    pub fields: Option<Vec<FieldDef>>,
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
    pub program_id: String, // Program ID as a string
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



// --------------------------------------------------
