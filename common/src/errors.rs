

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolifyError {
    #[error("IDL file not found: {0}")]
    IdlNotFound(String),

    #[error("Failed to parse IDL: {0}")]
    IdlParseFailed(String),

    #[error("Invalid instruction order: {0}")]
    InvalidInstructionOrder(String),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Instruction not found: {0}")]
    InstructionNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Template rendering error: {0}")]
    TemplateError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Dependency analysis failed: {0}")]
    DependencyAnalysisFailed(String),

    #[error("Invalid setup requirement")]
    InvalidSetupRequirement,

    #[error("Invalid PDA initialization")]
    InvalidPdaInitialization,

    #[error("Invalid test case")]
    InvalidTestCase,
    
    
}

pub type Result<T> = std::result::Result<T, SolifyError>;