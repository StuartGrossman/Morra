use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("FYG1SQaM49FGPQEGco8s9x4i3MzquotS48XcNre1UMBf");

#[program]
pub mod morra_program {
    use super::*;

    pub fn create_game(
        ctx: Context<CreateGame>,
        bet_amount: u64,
        commitment: [u8; 32],
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.creator = ctx.accounts.creator.key();
        game.bet_amount = bet_amount;
        game.creator_commitment = commitment;
        game.status = GameStatus::WaitingForOpponent;
        game.created_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn join_game(
        ctx: Context<JoinGame>,
        commitment: [u8; 32],
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::WaitingForOpponent, GameError::InvalidGameState);
        
        // Transfer bet amount from opponent to game account
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.opponent.to_account_info(),
                    to: ctx.accounts.game_account.to_account_info(),
                },
            ),
            game.bet_amount,
        )?;

        game.opponent = ctx.accounts.opponent.key();
        game.opponent_commitment = commitment;
        game.status = GameStatus::WaitingForReveal;
        Ok(())
    }

    pub fn reveal_move(
        ctx: Context<RevealMove>,
        card: u8,
        prediction: u8,
        salt: [u8; 32],
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        let player = &ctx.accounts.player;
        
        // Verify commitment
        let commitment = anchor_lang::solana_program::hash::hash(
            format!("{}{}{}", card, prediction, salt).as_bytes(),
        ).to_bytes();
        
        let is_creator = player.key() == game.creator;
        let expected_commitment = if is_creator {
            game.creator_commitment
        } else {
            game.opponent_commitment
        };
        
        require!(commitment == expected_commitment, GameError::InvalidCommitment);
        
        // Store move
        if is_creator {
            game.creator_card = Some(card);
            game.creator_prediction = Some(prediction);
        } else {
            game.opponent_card = Some(card);
            game.opponent_prediction = Some(prediction);
        }
        
        // Check if both players have revealed
        if game.creator_card.is_some() && game.opponent_card.is_some() {
            game.resolve_game()?;
        }
        
        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let game = &ctx.accounts.game;
        let winner = &ctx.accounts.winner;
        
        require!(game.status == GameStatus::Completed, GameError::InvalidGameState);
        require!(
            winner.key() == game.winner.unwrap(),
            GameError::NotWinner
        );
        
        // Transfer winnings to winner
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.game_account.to_account_info(),
                    to: ctx.accounts.winner.to_account_info(),
                },
                &[&[
                    b"game".as_ref(),
                    &game.key().to_bytes(),
                    &[*ctx.bumps.get("game_account").unwrap()],
                ]],
            ),
            game.bet_amount * 2,
        )?;
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bet_amount: u64)]
pub struct CreateGame<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Game::LEN,
        seeds = [b"game", creator.key().as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(
        mut,
        seeds = [b"game", game.creator.as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub opponent: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"game", game.creator.as_ref()],
        bump,
        constraint = game_account.key() == game.key()
    )]
    /// CHECK: This is the PDA that holds the game funds
    pub game_account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevealMove<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(
        mut,
        seeds = [b"game", game.creator.as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub winner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"game", game.creator.as_ref()],
        bump,
        constraint = game_account.key() == game.key()
    )]
    /// CHECK: This is the PDA that holds the game funds
    pub game_account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Game {
    pub creator: Pubkey,
    pub opponent: Pubkey,
    pub bet_amount: u64,
    pub creator_commitment: [u8; 32],
    pub opponent_commitment: [u8; 32],
    pub creator_card: Option<u8>,
    pub opponent_card: Option<u8>,
    pub creator_prediction: Option<u8>,
    pub opponent_prediction: Option<u8>,
    pub status: GameStatus,
    pub winner: Option<Pubkey>,
    pub created_at: i64,
}

impl Game {
    pub const LEN: usize = 32 + 32 + 8 + 32 + 32 + 1 + 1 + 1 + 1 + 1 + 32 + 8;

    pub fn resolve_game(&mut self) -> Result<()> {
        let creator_card = self.creator_card.unwrap();
        let opponent_card = self.opponent_card.unwrap();
        let creator_prediction = self.creator_prediction.unwrap();
        let opponent_prediction = self.opponent_prediction.unwrap();
        
        let total = creator_card + opponent_card;
        let creator_wins = creator_prediction == total;
        let opponent_wins = opponent_prediction == total;
        
        match (creator_wins, opponent_wins) {
            (true, false) => {
                self.winner = Some(self.creator);
                self.status = GameStatus::Completed;
            }
            (false, true) => {
                self.winner = Some(self.opponent);
                self.status = GameStatus::Completed;
            }
            _ => {
                self.status = GameStatus::Draw;
            }
        }
        
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    WaitingForOpponent,
    WaitingForReveal,
    Draw,
    Completed,
}

#[error_code]
pub enum GameError {
    #[msg("Invalid game state for this operation")]
    InvalidGameState,
    
    #[msg("Invalid commitment provided")]
    InvalidCommitment,
    
    #[msg("Only the winner can claim winnings")]
    NotWinner,
}
