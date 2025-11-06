use anchor_lang::prelude::*;
use crate::{
    state::user_config::UserConfig, 
    error::SolifyError, 
    types::IdlData, 
    analyzer::{hash_idl_data, DependencyAnalyzer},
    events::MetadataGenerated
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
        
        // Validate execution order
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

        msg!("Starting metadata generation for program: {}", program_name);
        msg!("Execution order: {:?}", execution_order);

        let analyzer = DependencyAnalyzer::new();
        let test_metadata = analyzer.analyze_dependencies(&idl_data, &execution_order)?;

        msg!("Dependency analysis completed successfully");
        msg!("Generated {} account dependencies", test_metadata.account_dependencies.len());
        msg!("Generated {} PDA initializations", test_metadata.pda_init_sequence.len());
        msg!("Generated {} setup requirements", test_metadata.setup_requirements.len());
        msg!("Generated {} test cases", test_metadata.test_cases.len());

        let idl_hash = hash_idl_data(&idl_data);
        
        self.user_config.update_after_generation(
            program_id.to_string(), 
            program_name.clone(), 
            idl_hash, 
            clock.unix_timestamp
        );

        emit!(MetadataGenerated {
            authority: self.authority.key(),
            program_id,
            program_name: program_name.clone(),
            idl_hash,
            instruction_count: idl_data.instructions.len() as u32,
            test_case_count: test_metadata.test_cases.len() as u32,
            timestamp: clock.unix_timestamp,
        });

        msg!("Metadata generation completed successfully for {}", program_name);
        
        Ok(())
    }
}