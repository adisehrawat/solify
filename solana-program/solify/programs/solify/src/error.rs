use anchor_lang::prelude::*;

#[error_code]
pub enum SolifyError {
    #[msg("Invalid instruction order")]
    InvalidInstructionOrder,
    #[msg("Circular dependency detected")]
    CircularDependency,
    #[msg("Invalid IDL data")]
    InvalidIdlData,
    #[msg("Invalid execution order")]
    InvalidExecutionOrder,
    #[msg("Invalid account dependency")]
    InvalidAccountDependency,
    #[msg("Invalid setup requirement")]
    InvalidSetupRequirement,
    #[msg("Invalid test case")]
    InvalidTestCase,
    #[msg("Invalid PDA detected")]
    InvalidPdaDetected,
    #[msg("Invalid account constraint parse failed")]
    AccountConstraintParseFailed,
    #[msg("Dependency analysis failed")]
    DependencyAnalysisFailed,
}

