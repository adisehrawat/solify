use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction as SolanaInstruction;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signature, Signer},
    transaction::Transaction,
};

use solify_common::types::{IdlData as CommonIdlData, TestMetadata as CommonTestMetadata};
use solify_common::ArgumentType as C;
use types::ArgumentType as T;
use std::str::FromStr;

#[path = "clients/rust/src/generated/mod.rs"]
pub mod generated;

pub use generated::programs::SOLIFY_ID;
pub use generated::{accounts, errors, instructions, types};

pub struct SolifyClient {
    rpc: RpcClient,
    commitment: CommitmentConfig,
}

impl SolifyClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self> {
        Self::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
    }

    pub fn new_with_commitment(
        rpc_url: impl AsRef<str>,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        let rpc = RpcClient::new_with_commitment(rpc_url.as_ref().to_string(), commitment);
        Ok(Self { rpc, commitment })
    }

    pub fn from_rpc_client(rpc: RpcClient, commitment: CommitmentConfig) -> Self {
        Self { rpc, commitment }
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.rpc
    }

    pub fn commitment(&self) -> CommitmentConfig {
        self.commitment
    }

    pub fn store_idl_data<S: Signer>(
        &self,
        authority: &S,
        program_id: Pubkey,
        idl_data: &CommonIdlData,
    ) -> Result<Signature> {
        let generated_idl = convert_idl_data(idl_data)?;
        let (idl_storage, _) = derive_idl_storage_address(&program_id, &authority.pubkey());

        let accounts = instructions::StoreIdlData {
            idl_storage,
            authority: authority.pubkey(),
            system_program: system_program_id(),
        };
        let args = instructions::StoreIdlDataInstructionArgs {
            idl_data: generated_idl,
            program_id,
        };
        let instruction = accounts.instruction(args);

        self.send_instruction(authority, &[instruction])
    }

    pub fn update_idl_data<S: Signer>(
        &self,
        authority: &S,
        program_id: Pubkey,
        idl_data: &CommonIdlData,
    ) -> Result<Signature> {
        let generated_idl = convert_idl_data(idl_data)?;
        let (idl_storage, _) = derive_idl_storage_address(&program_id, &authority.pubkey());

        let accounts = instructions::UpdateIdlData {
            idl_storage,
            authority: authority.pubkey(),
            system_program: system_program_id(),
        };
        let args = instructions::UpdateIdlDataInstructionArgs {
            idl_data: generated_idl,
            program_id,
        };
        let instruction = accounts.instruction(args);

        self.send_instruction(authority, &[instruction])
    }

    pub fn generate_metadata<S: Signer>(
        &self,
        authority: &S,
        program_id: Pubkey,
        execution_order: Vec<String>,
        paraphrase: &str,
        program_name: impl Into<String>,
    ) -> Result<Signature> {
        let (idl_storage, _) = derive_idl_storage_address(&program_id, &authority.pubkey());
        let (test_metadata_config, _) =
            derive_test_metadata_config_address(&program_id, &authority.pubkey(), paraphrase);

        let accounts = instructions::GenerateMetadata {
            test_metadata_config,
            idl_storage,
            authority: authority.pubkey(),
            system_program: system_program_id(),
        };
        let args = instructions::GenerateMetadataInstructionArgs {
            execution_order,
            program_id,
            program_name: program_name.into(),
            paraphrase: paraphrase.to_string(),
        };
        let instruction = accounts.instruction(args);

        self.send_instruction(authority, &[instruction])
    }



    pub fn fetch_idl_storage(
        &self,
        authority: Pubkey,
        program_id: Pubkey,
    ) -> Result<Option<IdlStorageAccount>> {
        let (address, _) = derive_idl_storage_address(&program_id, &authority);
        let response = self
            .rpc
            .get_account_with_commitment(&address, self.commitment)
            .context("Failed to fetch IDL storage account")?;

        if let Some(account) = response.value {
            let decoded = accounts::idl_storage::IdlStorage::from_bytes(&account.data)
                .context("Failed to decode IDL storage account data")?;
            let idl_data = convert_idl_data_back(&decoded.idl_data);

            Ok(Some(IdlStorageAccount {
                address,
                authority: decoded.authority,
                program_id: decoded.program_id,
                idl_data,
                timestamp: decoded.timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn fetch_test_metadata(
        &self,
        authority: Pubkey,
        program_id: Pubkey,
        paraphrase: &str,
    ) -> Result<Option<TestMetadataAccount>> {
        let (address, _) = derive_test_metadata_config_address(&program_id, &authority, &paraphrase);
        let response = self
            .rpc
            .get_account_with_commitment(&address, self.commitment)
            .context("Failed to fetch test metadata account")?;

        if let Some(account) = response.value {
            let decoded =
                accounts::test_metadata_config::TestMetadataConfig::from_bytes(&account.data)
                    .context("Failed to decode TestMetadataConfig account data")?;
            let test_metadata = convert_test_metadata_back(&decoded.test_metadata)?;

            Ok(Some(TestMetadataAccount {
                address,
                authority: decoded.authority,
                program_id: decoded.program_id,
                program_name: decoded.program_name,
                test_metadata,
                timestamp: decoded.timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    fn send_instruction<S: Signer>(
        &self,
        authority: &S,
        instructions: &[SolanaInstruction],
    ) -> Result<Signature> {
        let recent_blockhash = self
            .rpc
            .get_latest_blockhash()
            .context("Failed to fetch latest blockhash")?;

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&authority.pubkey()),
            &[authority],
            recent_blockhash,
        );

        // Simulate the transaction first to catch errors early
        let simulation_result = self.rpc.simulate_transaction(&transaction);
        if let Ok(simulation) = simulation_result {
            if let Some(err) = simulation.value.err {
                return Err(anyhow::anyhow!(
                    "Transaction simulation failed: {:?}. Logs: {:?}",
                    err,
                    simulation.value.logs
                ));
            }
        }

        // Send and confirm the transaction
        self.rpc
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                self.commitment,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to send Solify transaction: {}. \
                    This could be due to: insufficient funds, network issues, \
                    or program execution errors. Check your wallet balance and RPC connection.",
                    e
                )
            })
    }
}

pub fn derive_idl_storage_address(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"idl_storage", program_id.as_ref(), authority.as_ref()],
        &generated::SOLIFY_ID,
    )
}

pub fn derive_test_metadata_config_address(
    program_id: &Pubkey,
    authority: &Pubkey,
    paraphrase: &str,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"tests_metadata", program_id.as_ref(), authority.as_ref(), paraphrase.as_bytes()],
        &generated::SOLIFY_ID,
    )
}

#[derive(Debug, Clone)]
pub struct IdlStorageAccount {
    pub address: Pubkey,
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub idl_data: CommonIdlData,
    pub timestamp: i64,
}


#[derive(Debug, Clone)]
pub struct TestMetadataAccount {
    pub address: Pubkey,
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub program_name: String,
    pub test_metadata: CommonTestMetadata,
    pub timestamp: i64,
}


pub fn convert_idl_data(common: &solify_common::IdlData) -> Result<types::IdlData> {
    Ok(types::IdlData {
        name: common.name.clone(),
        version: common.version.clone(),
        instructions: common
            .instructions
            .iter()
            .map(convert_idl_instruction)
            .collect::<Result<Vec<_>>>()?,
        accounts: common
            .accounts
            .iter()
            .map(convert_idl_account)
            .collect::<Result<Vec<_>>>()?,
        types: common
            .types
            .iter()
            .map(convert_idl_typedef)
            .collect::<Result<Vec<_>>>()?,
        errors: common
            .errors
            .iter()
            .map(convert_idl_error)
            .collect::<Result<Vec<_>>>()?,
        constants: common
            .constants
            .iter()
            .map(convert_idl_constant)
            .collect::<Result<Vec<_>>>()?,
        events: common
            .events
            .iter()
            .map(convert_idl_event)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_idl_instruction(src: &solify_common::IdlInstruction) -> Result<types::IdlInstruction> {
    Ok(types::IdlInstruction {
        name: src.name.clone(),
        accounts: src
            .accounts
            .iter()
            .map(convert_idl_account_item)
            .collect::<Result<Vec<_>>>()?,
        args: src
            .args
            .iter()
            .map(convert_idl_field)
            .collect::<Result<Vec<_>>>()?,
        docs: src.docs.clone(),
    })
}

fn convert_idl_account_item(src: &solify_common::IdlAccountItem) -> Result<types::IdlAccountItem> {
    Ok(types::IdlAccountItem {
        name: src.name.clone(),
        is_mut: src.is_mut,
        is_signer: src.is_signer,
        is_optional: src.is_optional,
        docs: src.docs.clone(),
        pda: match &src.pda {
            Some(p) => Some(convert_idl_pda(p)?),
            None => None,
        },
    })
}

fn convert_idl_pda(src: &solify_common::IdlPda) -> Result<types::IdlPda> {
    Ok(types::IdlPda {
        seeds: src
            .seeds
            .iter()
            .map(convert_idl_seed)
            .collect::<Result<Vec<_>>>()?,
        // on-chain uses Option<String> for program in your anchor defs
        program: if src.program.is_empty() {
            None
        } else {
            Some(src.program.clone())
        },
    })
}

fn convert_idl_seed(src: &solify_common::IdlSeed) -> Result<types::IdlSeed> {
    Ok(types::IdlSeed {
        kind: src.kind.clone(),
        path: src.path.clone(),
        value: src.value.clone(),
    })
}

fn convert_idl_account(src: &solify_common::IdlAccount) -> Result<types::IdlAccount> {
    Ok(types::IdlAccount {
        name: src.name.clone(),
        fields: src
            .fields
            .iter()
            .map(convert_idl_field)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_idl_field(src: &solify_common::IdlField) -> Result<types::IdlField> {
    Ok(types::IdlField {
        name: src.name.clone(),
        field_type: src.field_type.clone(),
    })
}

fn convert_idl_typedef(src: &solify_common::IdlTypeDef) -> Result<types::IdlTypeDef> {
    Ok(types::IdlTypeDef {
        name: src.name.clone(),
        kind: src.kind.clone(),
        fields: src.fields.clone(),
    })
}

fn convert_idl_error(src: &solify_common::IdlError) -> Result<types::IdlError> {
    Ok(types::IdlError {
        code: src.code,
        name: src.name.clone(),
        msg: src.msg.clone(),
    })
}

fn convert_idl_constant(src: &solify_common::IdlConstant) -> Result<types::IdlConstant> {
    Ok(types::IdlConstant {
        name: src.name.clone(),
        constant_type: src.constant_type.clone(),
        value: src.value.clone(),
    })
}

fn convert_idl_event(src: &solify_common::IdlEvent) -> Result<types::IdlEvent> {
    Ok(types::IdlEvent {
        name: src.name.clone(),
        discriminator: src.discriminator.clone(),
        fields: src
            .fields
            .iter()
            .map(convert_idl_field)
            .collect::<Result<Vec<_>>>()?,
    })
}

// Convert from generated types back to common types for IdlData
fn convert_idl_data_back(generated: &types::IdlData) -> CommonIdlData {
    CommonIdlData {
        name: generated.name.clone(),
        version: generated.version.clone(),
        instructions: generated.instructions.iter().map(convert_idl_instruction_back).collect(),
        accounts: generated.accounts.iter().map(convert_idl_account_back).collect(),
        types: generated.types.iter().map(convert_idl_type_def_back).collect(),
        errors: generated.errors.iter().map(convert_idl_error_back).collect(),
        constants: generated.constants.iter().map(convert_idl_constant_back).collect(),
        events: generated.events.iter().map(convert_idl_event_back).collect(),
    }
}

fn convert_idl_instruction_back(generated: &types::IdlInstruction) -> solify_common::IdlInstruction {
    solify_common::IdlInstruction {
        name: generated.name.clone(),
        accounts: generated.accounts.iter().map(convert_idl_account_item_back).collect(),
        args: generated.args.iter().map(convert_idl_field_back).collect(),
        docs: generated.docs.clone(),
    }
}

fn convert_idl_account_item_back(generated: &types::IdlAccountItem) -> solify_common::IdlAccountItem {
    solify_common::IdlAccountItem {
        name: generated.name.clone(),
        is_mut: generated.is_mut,
        is_signer: generated.is_signer,
        is_optional: generated.is_optional,
        docs: generated.docs.clone(),
        pda: generated.pda.as_ref().map(convert_idl_pda_back),
    }
}

fn convert_idl_pda_back(generated: &types::IdlPda) -> solify_common::IdlPda {
    solify_common::IdlPda {
        seeds: generated.seeds.iter().map(convert_idl_seed_back).collect(),
        program: generated.program.clone().unwrap_or_default(),
    }
}

fn convert_idl_seed_back(generated: &types::IdlSeed) -> solify_common::IdlSeed {
    solify_common::IdlSeed {
        kind: generated.kind.clone(),
        path: generated.path.clone(),
        value: generated.value.clone(),
    }
}

fn convert_idl_field_back(generated: &types::IdlField) -> solify_common::IdlField {
    solify_common::IdlField {
        name: generated.name.clone(),
        field_type: generated.field_type.clone(),
    }
}

fn convert_idl_account_back(generated: &types::IdlAccount) -> solify_common::IdlAccount {
    solify_common::IdlAccount {
        name: generated.name.clone(),
        fields: generated.fields.iter().map(convert_idl_field_back).collect(),
    }
}

fn convert_idl_type_def_back(generated: &types::IdlTypeDef) -> solify_common::IdlTypeDef {
    solify_common::IdlTypeDef {
        name: generated.name.clone(),
        kind: generated.kind.clone(),
        fields: generated.fields.clone(),
    }
}

fn convert_idl_error_back(generated: &types::IdlError) -> solify_common::IdlError {
    solify_common::IdlError {
        code: generated.code,
        name: generated.name.clone(),
        msg: generated.msg.clone(),
    }
}

fn convert_idl_constant_back(generated: &types::IdlConstant) -> solify_common::IdlConstant {
    solify_common::IdlConstant {
        name: generated.name.clone(),
        constant_type: generated.constant_type.clone(),
        value: generated.value.clone(),
    }
}

fn convert_idl_event_back(generated: &types::IdlEvent) -> solify_common::IdlEvent {
    solify_common::IdlEvent {
        name: generated.name.clone(),
        discriminator: generated.discriminator.clone(),
        fields: generated.fields.iter().map(convert_idl_field_back).collect(),
    }
}

// ---------- TestMetadata conversion ----------

pub fn convert_test_metadata(src: &CommonTestMetadata) -> Result<types::TestMetadata> {
    Ok(types::TestMetadata {
        instruction_order: src.instruction_order.clone(),
        account_dependencies: src
            .account_dependencies
            .iter()
            .map(convert_account_dependency)
            .collect::<Result<Vec<_>>>()?,
        pda_init_sequence: src
            .pda_init_sequence
            .iter()
            .map(convert_pda_init)
            .collect::<Result<Vec<_>>>()?,
        setup_requirements: src
            .setup_requirements
            .iter()
            .map(convert_setup_requirement)
            .collect::<Result<Vec<_>>>()?,
        test_cases: src
            .test_cases
            .iter()
            .map(convert_instruction_test_cases)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_account_dependency(src: &solify_common::AccountDependency) -> Result<types::AccountDependency> {
    Ok(types::AccountDependency {
        account_name: src.account_name.clone(),
        depends_on: src.depends_on.clone(),
        is_pda: src.is_pda,
        is_signer: src.is_signer,
        is_mut: src.is_mut,
        must_be_initialized: src.must_be_initialized,
        initialization_order: src.initialization_order,
    })
}

fn convert_pda_init(src: &solify_common::PdaInit) -> Result<types::PdaInit> {
    // convert program_id string -> Pubkey
    let program_id = Pubkey::from_str(&src.program_id)
        .with_context(|| format!("Failed to parse program id '{}'", src.program_id))?;

    Ok(types::PdaInit {
        account_name: src.account_name.clone(),
        seeds: src
            .seeds
            .iter()
            .map(convert_seed_component)
            .collect::<Result<Vec<_>>>()?,
        program_id,
        space: src.space,
    })
}

fn convert_seed_component(src: &solify_common::SeedComponent) -> Result<types::SeedComponent> {
    Ok(types::SeedComponent {
        seed_type: match src.seed_type {
            solify_common::SeedType::Static => types::SeedType::Static,
            solify_common::SeedType::AccountKey => types::SeedType::AccountKey,
            solify_common::SeedType::Argument => types::SeedType::Argument,
        },
        value: src.value.clone(),
    })
}

fn convert_setup_requirement(src: &solify_common::SetupRequirement) -> Result<types::SetupRequirement> {
    Ok(types::SetupRequirement {
        requirement_type: match src.requirement_type {
            solify_common::SetupType::CreateKeypair => types::SetupType::CreateKeypair,
            solify_common::SetupType::FundAccount => types::SetupType::FundAccount,
            solify_common::SetupType::InitializePda => types::SetupType::InitializePda,
            solify_common::SetupType::MintTokens => types::SetupType::MintTokens,
            solify_common::SetupType::CreateAta => types::SetupType::CreateAta,
        },
        description: src.description.clone(),
        dependencies: src.dependencies.clone(),
    })
}

fn convert_instruction_test_cases(src: &solify_common::InstructionTestCases) -> Result<types::InstructionTestCases> {
    Ok(types::InstructionTestCases {
        instruction_name: src.instruction_name.clone(),
        arguments: src
            .arguments
            .iter()
            .map(convert_argument_info)
            .collect::<Result<Vec<_>>>()?,
        positive_cases: src
            .positive_cases
            .iter()
            .map(convert_test_case)
            .collect::<Result<Vec<_>>>()?,
        negative_cases: src
            .negative_cases
            .iter()
            .map(convert_test_case)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_argument_info(src: &solify_common::ArgumentInfo) -> Result<types::ArgumentInfo> {
    Ok(types::ArgumentInfo {
        name: src.name.clone(),
        arg_type: convert_argument_type(&src.arg_type)?,
        constraints: src.constraints.clone().into_iter().map(convert_constraint).collect::<Result<Vec<_>>>()?,
        is_optional: src.is_optional,
    })
}

fn convert_argument_type(src: &solify_common::ArgumentType) -> Result<types::ArgumentType> {
    // helper: produce a concise name string for an argument type (used for VecType/OptionType)
    fn arg_type_name(t: &C) -> Result<String> {
        match t {
            C::U8 => Ok("u8".to_string()),
            C::U16 => Ok("u16".to_string()),
            C::U32 => Ok("u32".to_string()),
            C::U64 => Ok("u64".to_string()),
            C::U128 => Ok("u128".to_string()),
            C::I8 => Ok("i8".to_string()),
            C::I16 => Ok("i16".to_string()),
            C::I32 => Ok("i32".to_string()),
            C::I64 => Ok("i64".to_string()),
            C::I128 => Ok("i128".to_string()),
            C::Bool => Ok("bool".to_string()),
            C::String { .. } => Ok("String".to_string()),
            C::Pubkey => Ok("Pubkey".to_string()),
            C::Vec { inner_type, .. } => {
                // recursive: produce inner name and wrap in Vec<...>
                let inner = arg_type_name(inner_type)?;
                Ok(format!("Vec<{}>", inner))
            }
            C::Option { inner_type } => {
                let inner = arg_type_name(inner_type)?;
                Ok(format!("Option<{}>", inner))
            }
            C::Struct { name } => Ok(name.clone()),
            C::Enum { name, .. } => Ok(name.clone()),
        }
    }

    let out = match src {
        C::U8 => T::U8,
        C::U16 => T::U16,
        C::U32 => T::U32,
        C::U64 => T::U64,
        C::U128 => T::U128,
        C::I8 => T::I8,
        C::I16 => T::I16,
        C::I32 => T::I32,
        C::I64 => T::I64,
        C::I128 => T::I128,
        C::Bool => T::Bool,
        C::String { max_length } => T::String { max_length: *max_length },
        C::Pubkey => T::Pubkey,
        C::Vec { inner_type, max_length } => {
            // Generated enum uses VecType { inner_type_name: String, max_length: Option<u32> }
            let inner_name = arg_type_name(inner_type)?;
            T::VecType {
                inner_type_name: inner_name,
                max_length: *max_length,
            }
        }
        C::Option { inner_type } => {
            let inner_name = arg_type_name(inner_type)?;
            T::OptionType {
                inner_type_name: inner_name,
            }
        }
        C::Struct { name } => {
            // Generated type doesn't have Struct variant, return error
            anyhow::bail!("Struct types are not supported in generated ArgumentType: {}", name);
        }
        C::Enum { name, .. } => {
            // Generated type doesn't have Enum variant, return error
            anyhow::bail!("Enum types are not supported in generated ArgumentType: {}", name);
        }
    };

    Ok(out)
}

fn convert_constraint(src: solify_common::ArgumentConstraint) -> Result<types::ArgumentConstraint> {
    use solify_common::ArgumentConstraint as C;
    use types::ArgumentConstraint as T;

    let out = match src {
        C::Min { value } => T::Min { value },
        C::Max { value } => T::Max { value },
        C::Range { min, max } => T::Range { min, max },
        C::NonZero => T::NonZero,
        C::MaxLength { value } => T::MaxLength { value },
        C::MinLength { value } => T::MinLength { value },
        C::Custom { .. } => {
            // If your generated type has Custom variant with description, adapt accordingly.
            // Here we fallback to MaxLength 0 to avoid mismatch — better to extend generated types.
            return Err(anyhow::anyhow!("Custom constraint mapping not implemented"))
        }
    };

    Ok(out)
}

fn convert_test_case(src: &solify_common::TestCase) -> Result<types::TestCase> {
    Ok(types::TestCase {
        test_type: match src.test_type {
            solify_common::TestCaseType::Positive => types::TestCaseType::Positive,
            solify_common::TestCaseType::NegativeBoundary => types::TestCaseType::NegativeBoundary,
            solify_common::TestCaseType::NegativeType => types::TestCaseType::NegativeType,
            solify_common::TestCaseType::NegativeConstraint => types::TestCaseType::NegativeConstraint,
            solify_common::TestCaseType::NegativeNull => types::TestCaseType::NegativeNull,
            solify_common::TestCaseType::NegativeOverflow => types::TestCaseType::NegativeOverflow,
        },
        description: src.description.clone(),
        argument_values: src
            .argument_values
            .iter()
            .map(convert_test_argument_value)
            .collect::<Result<Vec<_>>>()?,
        expected_outcome: convert_expected_outcome(&src.expected_outcome)?,
    })
}

fn convert_test_argument_value(src: &solify_common::TestArgumentValue) -> Result<types::TestArgumentValue> {
    Ok(types::TestArgumentValue {
        argument_name: src.argument_name.clone(),
        value_type: match &src.value_type {
            solify_common::TestValueType::Valid { description } => types::TestValueType::Valid { description: description.clone() },
            solify_common::TestValueType::Invalid { description, reason } => types::TestValueType::Invalid { description: description.clone(), reason: reason.clone() },
        },
    })
}

fn convert_expected_outcome(src: &solify_common::ExpectedOutcome) -> Result<types::ExpectedOutcome> {
    Ok(match src {
        solify_common::ExpectedOutcome::Success { state_changes } => types::ExpectedOutcome::Success { state_changes: state_changes.clone() },
        solify_common::ExpectedOutcome::Failure { error_code, error_message } => types::ExpectedOutcome::Failure { error_code: error_code.clone(), error_message: error_message.clone() },
    })
}

// Convert from generated types back to common types for TestMetadata
fn convert_test_metadata_back(src: &types::TestMetadata) -> Result<CommonTestMetadata> {
    Ok(CommonTestMetadata {
        instruction_order: src.instruction_order.clone(),
        account_dependencies: src
            .account_dependencies
            .iter()
            .map(convert_account_dependency_back)
            .collect(),
        pda_init_sequence: src
            .pda_init_sequence
            .iter()
            .map(convert_pda_init_back)
            .collect::<Result<Vec<_>>>()?,
        setup_requirements: src
            .setup_requirements
            .iter()
            .map(convert_setup_requirement_back)
            .collect(),
        test_cases: src
            .test_cases
            .iter()
            .map(convert_instruction_test_cases_back)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_account_dependency_back(src: &types::AccountDependency) -> solify_common::AccountDependency {
    solify_common::AccountDependency {
        account_name: src.account_name.clone(),
        depends_on: src.depends_on.clone(),
        is_pda: src.is_pda,
        is_signer: src.is_signer,
        is_mut: src.is_mut,
        must_be_initialized: src.must_be_initialized,
        initialization_order: src.initialization_order,
    }
}

fn convert_pda_init_back(src: &types::PdaInit) -> Result<solify_common::PdaInit> {
    Ok(solify_common::PdaInit {
        account_name: src.account_name.clone(),
        seeds: src
            .seeds
            .iter()
            .map(convert_seed_component_back)
            .collect(),
        program_id: src.program_id.to_string(),
        space: src.space,
    })
}

fn convert_seed_component_back(src: &types::SeedComponent) -> solify_common::SeedComponent {
    solify_common::SeedComponent {
        seed_type: match src.seed_type {
            types::SeedType::Static => solify_common::SeedType::Static,
            types::SeedType::AccountKey => solify_common::SeedType::AccountKey,
            types::SeedType::Argument => solify_common::SeedType::Argument,
        },
        value: src.value.clone(),
    }
}

fn convert_setup_requirement_back(src: &types::SetupRequirement) -> solify_common::SetupRequirement {
    solify_common::SetupRequirement {
        requirement_type: match src.requirement_type {
            types::SetupType::CreateKeypair => solify_common::SetupType::CreateKeypair,
            types::SetupType::FundAccount => solify_common::SetupType::FundAccount,
            types::SetupType::InitializePda => solify_common::SetupType::InitializePda,
            types::SetupType::MintTokens => solify_common::SetupType::MintTokens,
            types::SetupType::CreateAta => solify_common::SetupType::CreateAta,
        },
        description: src.description.clone(),
        dependencies: src.dependencies.clone(),
    }
}

fn convert_instruction_test_cases_back(src: &types::InstructionTestCases) -> Result<solify_common::InstructionTestCases> {
    Ok(solify_common::InstructionTestCases {
        instruction_name: src.instruction_name.clone(),
        arguments: src
            .arguments
            .iter()
            .map(convert_argument_info_back)
            .collect::<Result<Vec<_>>>()?,
        positive_cases: src
            .positive_cases
            .iter()
            .map(convert_test_case_back)
            .collect::<Result<Vec<_>>>()?,
        negative_cases: src
            .negative_cases
            .iter()
            .map(convert_test_case_back)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_argument_info_back(src: &types::ArgumentInfo) -> Result<solify_common::ArgumentInfo> {
    Ok(solify_common::ArgumentInfo {
        name: src.name.clone(),
        arg_type: convert_argument_type_back(&src.arg_type)?,
        constraints: src.constraints.iter().map(convert_constraint_back).collect(),
        is_optional: src.is_optional,
    })
}

fn convert_argument_type_back(src: &types::ArgumentType) -> Result<solify_common::ArgumentType> {
    use types::ArgumentType as T;
    use solify_common::ArgumentType as C;

    let out = match src {
        T::U8 => C::U8,
        T::U16 => C::U16,
        T::U32 => C::U32,
        T::U64 => C::U64,
        T::U128 => C::U128,
        T::I8 => C::I8,
        T::I16 => C::I16,
        T::I32 => C::I32,
        T::I64 => C::I64,
        T::I128 => C::I128,
        T::Bool => C::Bool,
        T::String { max_length } => C::String { max_length: *max_length },
        T::Pubkey => C::Pubkey,
        T::VecType { inner_type_name, max_length } => {
            // Parse the inner type name back to ArgumentType
            let inner_type = parse_argument_type_from_name(inner_type_name)?;
            C::Vec {
                inner_type: Box::new(inner_type),
                max_length: *max_length,
            }
        }
        T::OptionType { inner_type_name } => {
            let inner_type = parse_argument_type_from_name(inner_type_name)?;
            C::Option {
                inner_type: Box::new(inner_type),
            }
        }
    };
    Ok(out)
}

fn parse_argument_type_from_name(name: &str) -> Result<solify_common::ArgumentType> {
    // Simple parser for basic types - this is a simplified version
    match name {
        "u8" => Ok(solify_common::ArgumentType::U8),
        "u16" => Ok(solify_common::ArgumentType::U16),
        "u32" => Ok(solify_common::ArgumentType::U32),
        "u64" => Ok(solify_common::ArgumentType::U64),
        "u128" => Ok(solify_common::ArgumentType::U128),
        "i8" => Ok(solify_common::ArgumentType::I8),
        "i16" => Ok(solify_common::ArgumentType::I16),
        "i32" => Ok(solify_common::ArgumentType::I32),
        "i64" => Ok(solify_common::ArgumentType::I64),
        "i128" => Ok(solify_common::ArgumentType::I128),
        "bool" => Ok(solify_common::ArgumentType::Bool),
        "String" => Ok(solify_common::ArgumentType::String { max_length: None }),
        "Pubkey" => Ok(solify_common::ArgumentType::Pubkey),
        _ => {
            // Try to parse Vec<...> or Option<...>
            if let Some(inner) = name.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
                let inner_type = parse_argument_type_from_name(inner)?;
                Ok(solify_common::ArgumentType::Vec {
                    inner_type: Box::new(inner_type),
                    max_length: None,
                })
            } else if let Some(inner) = name.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
                let inner_type = parse_argument_type_from_name(inner)?;
                Ok(solify_common::ArgumentType::Option {
                    inner_type: Box::new(inner_type),
                })
            } else {
                // For unknown types, treat as Struct
                Ok(solify_common::ArgumentType::Struct { name: name.to_string() })
            }
        }
    }
}

fn convert_constraint_back(src: &types::ArgumentConstraint) -> solify_common::ArgumentConstraint {
    use types::ArgumentConstraint as T;
    use solify_common::ArgumentConstraint as C;

    match src {
        T::Min { value } => C::Min { value: *value },
        T::Max { value } => C::Max { value: *value },
        T::Range { min, max } => C::Range { min: *min, max: *max },
        T::NonZero => C::NonZero,
        T::MaxLength { value } => C::MaxLength { value: *value },
        T::MinLength { value } => C::MinLength { value: *value }
    }
}

fn convert_test_case_back(src: &types::TestCase) -> Result<solify_common::TestCase> {
    Ok(solify_common::TestCase {
        test_type: match src.test_type {
            types::TestCaseType::Positive => solify_common::TestCaseType::Positive,
            types::TestCaseType::NegativeBoundary => solify_common::TestCaseType::NegativeBoundary,
            types::TestCaseType::NegativeType => solify_common::TestCaseType::NegativeType,
            types::TestCaseType::NegativeConstraint => solify_common::TestCaseType::NegativeConstraint,
            types::TestCaseType::NegativeNull => solify_common::TestCaseType::NegativeNull,
            types::TestCaseType::NegativeOverflow => solify_common::TestCaseType::NegativeOverflow,
        },
        description: src.description.clone(),
        argument_values: src.argument_values.iter().map(convert_test_argument_value_back).collect(),
        expected_outcome: convert_expected_outcome_back(&src.expected_outcome),
    })
}

fn convert_test_argument_value_back(src: &types::TestArgumentValue) -> solify_common::TestArgumentValue {
    solify_common::TestArgumentValue {
        argument_name: src.argument_name.clone(),
        value_type: match &src.value_type {
            types::TestValueType::Valid { description } => {
                solify_common::TestValueType::Valid { description: description.clone() }
            }
            types::TestValueType::Invalid { description, reason } => {
                solify_common::TestValueType::Invalid {
                    description: description.clone(),
                    reason: reason.clone(),
                }
            }
        },
    }
}

fn convert_expected_outcome_back(src: &types::ExpectedOutcome) -> solify_common::ExpectedOutcome {
    match src {
        types::ExpectedOutcome::Success { state_changes } => {
            solify_common::ExpectedOutcome::Success {
                state_changes: state_changes.clone(),
            }
        }
        types::ExpectedOutcome::Failure { error_code, error_message } => {
            solify_common::ExpectedOutcome::Failure {
                error_code: error_code.clone(),
                error_message: error_message.clone(),
            }
        }
    }
}

#[inline]
fn system_program_id() -> Pubkey {
    Pubkey::from_str("11111111111111111111111111111111").unwrap()
}
