import { PublicKey } from '@solana/web3.js';
import { WalletContextState } from '@solana/wallet-adapter-react';

export interface GameState {
  id: string;
  creator: string;
  opponent: string | null;
  betAmount: number;
  creatorCard: number | null;
  opponentCard: number | null;
  creatorPrediction: number | null;
  opponentPrediction: number | null;
  creatorCommitment: string | null;
  opponentCommitment: string | null;
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
  connection: any;
  wallet: WalletContextState;
} 