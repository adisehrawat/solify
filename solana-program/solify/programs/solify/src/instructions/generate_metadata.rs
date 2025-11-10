use anchor_lang::prelude::*;
use crate::{
    state::user_config::UserConfig, 
    error::SolifyError, 
    types::IdlData, 
    analyzer::{hash_idl_data, DependencyAnalyzer},
    events::{ TestMetadataGenerated, CompleteTestMetadata}
};

#[derive(Accounts)]
pub struct GenerateMetadata<'info> {
    #[account(
        mut,
        seeds = [b"user_config", authority.key().as_ref()],
        bump = user_config.bump
    )]
    pub user_config: Account<'info, UserConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> GenerateMetadata<'info> {
    pub fn generate_metadata(
        &mut self, 
        idl_data: IdlData,
        execution_order: Vec<String>,
        program_id: Pubkey,
        program_name: String,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        require!(
            !execution_order.is_empty(),
            SolifyError::InvalidInstructionOrder
        );

        require!(
            execution_order.len() <= idl_data.instructions.len(),
            SolifyError::InvalidInstructionOrder
        );

        for instruction_name in &execution_order {
            require!(
                idl_data.instructions.iter().any(|i| &i.name == instruction_name),
                SolifyError::InvalidInstructionOrder
            );
        }

        let analyzer = DependencyAnalyzer::new();
        let test_metadata = analyzer.analyze_dependencies(&idl_data, &execution_order, program_id.to_string())?;

        let positive_test_cases: u32 = test_metadata.test_cases
            .iter()
            .map(|tc| tc.positive_cases.len() as u32)
            .sum();
        
        let negative_test_cases: u32 = test_metadata.test_cases
            .iter()
            .map(|tc| tc.negative_cases.len() as u32)
            .sum();

        let idl_hash = hash_idl_data(&idl_data);
        
        self.user_config.update_after_generation(
            program_id.to_string(), 
            program_name.clone(), 
            idl_hash, 
            clock.unix_timestamp
        );

        emit!(CompleteTestMetadata {
            authority: self.authority.key(),
            program_id,
            program_name: program_name.clone(),
            test_metadata: test_metadata.to_event(),
            timestamp: clock.unix_timestamp,
        });

        emit!(TestMetadataGenerated {
            authority: self.authority.key(),
            program_id,
            program_name: program_name.clone(),
            account_dependencies_count: test_metadata.account_dependencies.len() as u32,
            pda_init_count: test_metadata.pda_init_sequence.len() as u32,
            setup_requirements_count: test_metadata.setup_requirements.len() as u32,
            total_test_cases: test_metadata.test_cases.len() as u32,
            positive_test_cases,
            negative_test_cases,
            timestamp: clock.unix_timestamp,
        });

        
        Ok(())
    }
}