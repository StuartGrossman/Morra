use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("FYG1SQaM49FGPQEGco8s9x4i3MzquotS48XcNre1UMBf");

#[program]
pub mod morra_program {
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, bet_amount: u64) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.creator = ctx.accounts.creator.key();
        game.bet_amount = bet_amount;
        game.status = GameStatus::WaitingForOpponent;
        game.winner = None;

        // Transfer bet amount from creator to game account
        let transfer_ix = system_program::Transfer {
            from: ctx.accounts.creator.to_account_info(),
            to: ctx.accounts.game.to_account_info(),
        };

        anchor_lang::solana_program::program::invoke(
            &transfer_ix.into(),
            &[
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.game.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn join_game(ctx: Context<JoinGame>, bet_amount: u64) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::WaitingForOpponent, GameError::GameNotAvailable);
        require!(game.bet_amount == bet_amount, GameError::InvalidBetAmount);

        game.opponent = Some(ctx.accounts.opponent.key());
        game.status = GameStatus::WaitingForMoves;

        // Transfer bet amount from opponent to game account
        let transfer_ix = system_program::Transfer {
            from: ctx.accounts.opponent.to_account_info(),
            to: ctx.accounts.game.to_account_info(),
        };

        anchor_lang::solana_program::program::invoke(
            &transfer_ix.into(),
            &[
                ctx.accounts.opponent.to_account_info(),
                ctx.accounts.game.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn submit_move(ctx: Context<SubmitMove>, commitment: [u8; 32]) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::WaitingForMoves, GameError::GameNotInProgress);

        if ctx.accounts.player.key() == game.creator {
            game.creator_commitment = Some(commitment);
        } else {
            require!(Some(ctx.accounts.player.key()) == game.opponent, GameError::InvalidPlayer);
            game.opponent_commitment = Some(commitment);
        }

        if game.creator_commitment.is_some() && game.opponent_commitment.is_some() {
            game.status = GameStatus::WaitingForReveal;
        }

        Ok(())
    }

    pub fn reveal_move(ctx: Context<RevealMove>, card: u8, prediction: u8, salt: [u8; 32]) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.status == GameStatus::WaitingForReveal, GameError::GameNotInProgress);
        require!(card >= 1 && card <= 5, GameError::InvalidCard);
        require!(prediction >= 0 && prediction <= 10, GameError::InvalidPrediction);

        let commitment = if ctx.accounts.player.key() == game.creator {
            game.creator_commitment
        } else {
            require!(Some(ctx.accounts.player.key()) == game.opponent, GameError::InvalidPlayer);
            game.opponent_commitment
        };

        require!(commitment.is_some(), GameError::NoCommitmentFound);

        // Verify commitment
        let mut data = Vec::new();
        data.extend_from_slice(&[card]);
        data.extend_from_slice(&[prediction]);
        data.extend_from_slice(&salt);
        let computed_hash = anchor_lang::solana_program::hash::hash(&data).to_bytes();

        require!(
            computed_hash == commitment.unwrap(),
            GameError::InvalidCommitment
        );

        if ctx.accounts.player.key() == game.creator {
            game.creator_card = Some(card);
            game.creator_prediction = Some(prediction);
        } else {
            game.opponent_card = Some(card);
            game.opponent_prediction = Some(prediction);
        }

        if game.creator_card.is_some() && game.opponent_card.is_some() {
            game.status = GameStatus::Completed;
            game.winner = Some(determine_winner(
                game.creator_card.unwrap(),
                game.creator_prediction.unwrap(),
                game.opponent_card.unwrap(),
                game.opponent_prediction.unwrap(),
                game.creator,
                game.opponent.unwrap(),
            ));
        }

        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        let game = &ctx.accounts.game;
        require!(game.status == GameStatus::Completed, GameError::GameNotCompleted);
        require!(game.winner.is_some(), GameError::NoWinner);

        let winner = game.winner.unwrap();
        let winner_account = if winner == game.creator {
            &ctx.accounts.creator
        } else {
            &ctx.accounts.opponent
        };

        let transfer_ix = system_program::Transfer {
            from: ctx.accounts.game.to_account_info(),
            to: winner_account.to_account_info(),
        };

        anchor_lang::solana_program::program::invoke(
            &transfer_ix.into(),
            &[
                ctx.accounts.game.to_account_info(),
                winner_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
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
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub opponent: Signer<'info>,
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
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    /// CHECK: This is safe because we verify the winner in the instruction
    #[account(mut)]
    pub creator: AccountInfo<'info>,
    /// CHECK: This is safe because we verify the winner in the instruction
    #[account(mut)]
    pub opponent: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Game {
    pub creator: Pubkey,
    pub opponent: Option<Pubkey>,
    pub bet_amount: u64,
    pub creator_commitment: Option<[u8; 32]>,
    pub opponent_commitment: Option<[u8; 32]>,
    pub creator_card: Option<u8>,
    pub opponent_card: Option<u8>,
    pub creator_prediction: Option<u8>,
    pub opponent_prediction: Option<u8>,
    pub status: GameStatus,
    pub winner: Option<Pubkey>,
}

impl Game {
    pub const LEN: usize = 32 + 33 + 8 + 33 + 33 + 2 + 2 + 2 + 2 + 1 + 33;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    WaitingForOpponent,
    WaitingForMoves,
    WaitingForReveal,
    Completed,
}

#[error_code]
pub enum GameError {
    #[msg("Game is not available")]
    GameNotAvailable,
    #[msg("Invalid bet amount")]
    InvalidBetAmount,
    #[msg("Game is not in progress")]
    GameNotInProgress,
    #[msg("Invalid player")]
    InvalidPlayer,
    #[msg("No commitment found")]
    NoCommitmentFound,
    #[msg("Invalid commitment")]
    InvalidCommitment,
    #[msg("Game is not completed")]
    GameNotCompleted,
    #[msg("No winner")]
    NoWinner,
    #[msg("Invalid card")]
    InvalidCard,
    #[msg("Invalid prediction")]
    InvalidPrediction,
}

fn determine_winner(
    creator_card: u8,
    creator_prediction: u8,
    opponent_card: u8,
    opponent_prediction: u8,
    creator: Pubkey,
    opponent: Pubkey,
) -> Pubkey {
    let total = creator_card + opponent_card;
    let creator_correct = creator_prediction == total;
    let opponent_correct = opponent_prediction == total;

    match (creator_correct, opponent_correct) {
        (true, false) => creator,
        (false, true) => opponent,
        (true, true) => {
            if creator_card > opponent_card {
                creator
            } else {
                opponent
            }
        }
        (false, false) => {
            if creator_card > opponent_card {
                creator
            } else {
                opponent
            }
        }
    }
}
