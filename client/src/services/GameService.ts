import { PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { db } from '../config/firebase';
import {
  doc,
  setDoc,
  getDoc,
  updateDoc,
  onSnapshot
} from 'firebase/firestore';
import { GameState, GameConfig } from '../types/game';

export class GameService {
  private program: Program;

  constructor(config: GameConfig) {
    if (!(config.programId instanceof PublicKey)) {
      throw new Error('Invalid program ID');
    }
    const provider = new AnchorProvider(
      config.connection,
      config.wallet as any, // Type assertion needed due to wallet adapter types
      { commitment: 'confirmed' }
    );
    this.program = new Program(
      {
        version: '0.1.0',
        name: 'morra_game',
        instructions: []
      },
      config.programId,
      provider
    );
  }

  async createGame(): Promise<string> {
    const gameRef = doc(db, 'games', Date.now().toString());
    const initialState: GameState = {
      creator: this.program.provider.publicKey?.toString() || '',
      opponent: null,
      betAmount: 0,
      creatorCard: null,
      opponentCard: null,
      creatorPrediction: null,
      opponentPrediction: null,
      creatorCommitment: null,
      opponentCommitment: null,
      status: 'waiting',
      winner: null
    };

    await setDoc(gameRef, initialState);
    return gameRef.id;
  }

  async joinGame(gameId: string): Promise<void> {
    const gameRef = doc(db, 'games', gameId);
    const gameDoc = await getDoc(gameRef);
    
    if (!gameDoc.exists()) {
      throw new Error('Game not found');
    }

    const gameData = gameDoc.data() as GameState;
    if (gameData.opponent) {
      throw new Error('Game is full');
    }

    await updateDoc(gameRef, {
      opponent: this.program.provider.publicKey?.toString() || '',
      status: 'in_progress'
    });
  }

  async submitMove(gameId: string, commitment: { card: number; prediction: number }): Promise<void> {
    const gameRef = doc(db, 'games', gameId);
    const gameDoc = await getDoc(gameRef);
    
    if (!gameDoc.exists()) {
      throw new Error('Game not found');
    }

    const gameData = gameDoc.data() as GameState;
    const isCreator = gameData.creator === this.program.provider.publicKey?.toString();

    if (isCreator) {
      await updateDoc(gameRef, {
        creatorCommitment: commitment
      });
    } else {
      await updateDoc(gameRef, {
        opponentCommitment: commitment
      });
    }
  }

  async revealMove(gameId: string, card: number, prediction: number): Promise<void> {
    const gameRef = doc(db, 'games', gameId);
    const gameDoc = await getDoc(gameRef);
    
    if (!gameDoc.exists()) {
      throw new Error('Game not found');
    }

    const gameData = gameDoc.data() as GameState;
    const isCreator = gameData.creator === this.program.provider.publicKey?.toString();

    if (isCreator) {
      await updateDoc(gameRef, {
        creatorCard: card,
        creatorPrediction: prediction,
        status: gameData.opponentCard !== null ? 'completed' : 'in_progress'
      });
    } else {
      await updateDoc(gameRef, {
        opponentCard: card,
        opponentPrediction: prediction,
        status: gameData.creatorCard !== null ? 'completed' : 'in_progress'
      });
    }

    if (gameData.creatorCard && gameData.opponentCard) {
      const winner = this.determineWinner(
        gameData.creatorCard,
        gameData.creatorPrediction || 0,
        gameData.opponentCard,
        gameData.opponentPrediction || 0
      );
      
      await updateDoc(gameRef, {
        winner,
        status: 'completed'
      });
    }
  }

  subscribeToGameState(gameId: string, callback: (state: GameState) => void): () => void {
    const gameRef = doc(db, 'games', gameId);
    return onSnapshot(gameRef, (doc) => {
      if (doc.exists()) {
        callback(doc.data() as GameState);
      }
    });
  }

  private determineWinner(
    card1: number,
    prediction1: number,
    card2: number,
    prediction2: number
  ): string | null {
    const total = card1 + card2;

    if (prediction1 === total && prediction2 === total) {
      return null; // Draw
    } else if (prediction1 === total) {
      return 'creator';
    } else if (prediction2 === total) {
      return 'opponent';
    } else {
      return null;
    }
  }
} 