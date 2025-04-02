use anchor_lang::prelude::*;

declare_id!("BD2SUgjKn9LivUUCrF5QNVx9ZC8xNKdFQovy8C24h49F");

#[program]
pub mod morra_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
