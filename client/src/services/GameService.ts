import { GameState, GameCommitment } from '../types/game';

export class GameService {
  private gameState: GameState | null = null;

  constructor() {
    // Initialize with empty state
    this.gameState = {
      id: Math.random().toString(36).substring(7),
      creator: 'Player 1',
      opponent: null,
      betAmount: 0,
      creatorCard: null,
      opponentCard: null,
      creatorPrediction: null,
      opponentPrediction: null,
      creatorCommitment: null,
      opponentCommitment: null,
      status: 'waiting',
      winner: null,
    };
  }

  async createGame(): Promise<string> {
    return this.gameState!.id;
  }

  async joinGame(gameId: string): Promise<void> {
    if (this.gameState!.id !== gameId) {
      throw new Error('Invalid game ID');
    }
    this.gameState!.opponent = 'Player 2';
    this.gameState!.status = 'in_progress';
  }

  async submitMove(gameId: string, commitment: GameCommitment): Promise<void> {
    if (this.gameState!.id !== gameId) {
      throw new Error('Invalid game ID');
    }

    const commitmentHash = await this.generateCommitment(commitment);
    
    if (!this.gameState!.creatorCommitment) {
      this.gameState!.creatorCommitment = commitmentHash;
    } else {
      this.gameState!.opponentCommitment = commitmentHash;
    }
  }

  async revealMove(gameId: string, card: number, prediction: number, salt: string): Promise<void> {
    if (this.gameState!.id !== gameId) {
      throw new Error('Invalid game ID');
    }

    if (!this.gameState!.creatorCard) {
      this.gameState!.creatorCard = card;
      this.gameState!.creatorPrediction = prediction;
    } else {
      this.gameState!.opponentCard = card;
      this.gameState!.opponentPrediction = prediction;
      this.determineWinner();
    }
  }

  async getGameState(gameId: string): Promise<GameState> {
    if (this.gameState!.id !== gameId) {
      throw new Error('Invalid game ID');
    }
    return this.gameState!;
  }

  private async generateCommitment(commitment: GameCommitment): Promise<string> {
    const data = `${commitment.card}${commitment.prediction}${commitment.salt}`;
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  private determineWinner(): void {
    const { creatorCard, creatorPrediction, opponentCard, opponentPrediction } = this.gameState!;
    const total = (creatorCard || 0) + (opponentCard || 0);
    
    if (creatorPrediction === total && opponentPrediction !== total) {
      this.gameState!.winner = this.gameState!.creator;
    } else if (opponentPrediction === total && creatorPrediction !== total) {
      this.gameState!.winner = this.gameState!.opponent;
    }
    
    this.gameState!.status = 'completed';
  }
} 