use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("YOUR_PROGRAM_ID"); // Replace with actual program ID

#[program]
pub mod morra {
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, bet: u64) -> Result<()> {
        // Validate bet amount
        require!(bet >= MIN_BET && bet <= MAX_BET, ErrorCode::InvalidBetAmount);

        let game = &mut ctx.accounts.game;
        game.creator = ctx.accounts.creator.key();
        game.bet = bet;
        game.status = GameStatus::Waiting;
        game.created_at = Clock::get()?.unix_timestamp;
        game.last_action_at = Clock::get()?.unix_timestamp;

        // Transfer bet amount to program account
        anchor_lang::solana_program::program::invoke(
            &system_program::Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.game.to_account_info(),
                lamports: bet,
            },
            &[
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.game.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn join_game(ctx: Context<JoinGame>, bet: u64) -> Result<()> {
        // Validate bet amount matches game bet
        require!(bet == ctx.accounts.game.bet, ErrorCode::InvalidBetAmount);

        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::Waiting, ErrorCode::GameNotAvailable);
        require!(game.creator != ctx.accounts.player.key(), ErrorCode::CannotJoinOwnGame);

        game.player = ctx.accounts.player.key();
        game.status = GameStatus::Playing;
        game.last_action_at = Clock::get()?.unix_timestamp;

        // Transfer bet amount to program account
        anchor_lang::solana_program::program::invoke(
            &system_program::Transfer {
                from: ctx.accounts.player.to_account_info(),
                to: ctx.accounts.game.to_account_info(),
                lamports: bet,
            },
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.game.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn submit_move(ctx: Context<SubmitMove>, commitment_hash: [u8; 32]) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::Playing, ErrorCode::GameNotInProgress);
        
        let player = if ctx.accounts.player.key() == game.creator {
            &mut game.creator_move
        } else {
            &mut game.player_move
        };

        player.committed_hash = commitment_hash;
        game.last_action_at = Clock::get()?.unix_timestamp;

        // Check if both players have submitted moves
        if game.creator_move.committed_hash.is_some() && game.player_move.committed_hash.is_some() {
            game.status = GameStatus::Revealing;
        }

        Ok(())
    }

    pub fn reveal_move(
        ctx: Context<RevealMove>,
        card: u8,
        prediction: u8,
        salt: [u8; 32],
    ) -> Result<()> {
        // Validate card and prediction
        require!(card >= 1 && card <= 5, ErrorCode::InvalidCard);
        require!(prediction >= 2 && prediction <= 10, ErrorCode::InvalidPrediction);

        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::Revealing, ErrorCode::GameNotInRevealPhase);

        let player = if ctx.accounts.player.key() == game.creator {
            &mut game.creator_move
        } else {
            &mut game.player_move
        };

        // Verify commitment
        let mut hasher = anchor_lang::solana_program::hash::hash::hash;
        let mut data = Vec::new();
        data.extend_from_slice(&[card]);
        data.extend_from_slice(&[prediction]);
        data.extend_from_slice(&salt);
        let hash = hasher(data.as_slice());
        
        require!(
            hash.to_bytes() == player.committed_hash.unwrap(),
            ErrorCode::InvalidCommitment
        );

        player.card = Some(card);
        player.prediction = Some(prediction);
        game.last_action_at = Clock::get()?.unix_timestamp;

        // Check if both players have revealed moves
        if game.creator_move.card.is_some() && game.player_move.card.is_some() {
            // Calculate winner
            let total = game.creator_move.card.unwrap() + game.player_move.card.unwrap();
            let winner = if game.creator_move.prediction.unwrap() == total {
                game.creator
            } else if game.player_move.prediction.unwrap() == total {
                game.player
            } else {
                // No winner, refund bets
                game.status = GameStatus::Complete;
                return Ok(());
            };

            game.winner = Some(winner);
            game.status = GameStatus::Complete;

            // Transfer pot to winner
            let pot = game.bet * 2;
            **ctx.accounts.winner.try_borrow_mut_lamports()? += pot;
            **ctx.accounts.game.try_borrow_mut_lamports()? -= pot;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(
        init,
        payer = creator,
        space = Game::LEN,
        seeds = [b"game", creator.key().as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitMove<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevealMove<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    /// CHECK: This is the winner's account that will receive the pot
    #[account(mut)]
    pub winner: AccountInfo<'info>,
}

#[account]
pub struct Game {
    pub creator: Pubkey,
    pub player: Pubkey,
    pub bet: u64,
    pub status: GameStatus,
    pub creator_move: PlayerMove,
    pub player_move: PlayerMove,
    pub winner: Option<Pubkey>,
    pub created_at: i64,
    pub last_action_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum GameStatus {
    Waiting,
    Playing,
    Revealing,
    Complete,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlayerMove {
    pub committed_hash: Option<[u8; 32]>,
    pub card: Option<u8>,
    pub prediction: Option<u8>,
}

impl Game {
    pub const LEN: usize = 8 + // discriminator
        32 + // creator
        32 + // player
        8 + // bet
        1 + // status
        (1 + 32 + 1 + 1) * 2 + // moves
        (1 + 32) + // winner
        8 + // created_at
        8; // last_action_at
}

const MIN_BET: u64 = 100_000_000; // 0.1 SOL
const MAX_BET: u64 = 10_000_000_000; // 10 SOL

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid bet amount")]
    InvalidBetAmount,
    #[msg("Game is not available")]
    GameNotAvailable,
    #[msg("Cannot join your own game")]
    CannotJoinOwnGame,
    #[msg("Game is not in progress")]
    GameNotInProgress,
    #[msg("Game is not in reveal phase")]
    GameNotInRevealPhase,
    #[msg("Invalid card")]
    InvalidCard,
    #[msg("Invalid prediction")]
    InvalidPrediction,
    #[msg("Invalid commitment")]
    InvalidCommitment,
} 