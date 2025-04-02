import { Connection, PublicKey } from '@solana/web3.js';
import { WalletContextState } from '@solana/wallet-adapter-react';

export interface GameState {
  creator: string;
  opponent: string | null;
  betAmount: number;
  creatorCard: number | null;
  opponentCard: number | null;
  creatorPrediction: number | null;
  opponentPrediction: number | null;
  creatorCommitment: { card: number; prediction: number } | null;
  opponentCommitment: { card: number; prediction: number } | null;
  status: 'waiting' | 'in_progress' | 'completed';
  winner: string | null;
}

export interface GameCommitment {
  card: number;
  prediction: number;
  salt: string;
}

export interface GameConfig {
  programId: PublicKey;
  connection: Connection;
  wallet: WalletContextState;
} 