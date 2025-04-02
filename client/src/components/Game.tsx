import { useState, useEffect } from 'react';
import styled from 'styled-components';
import { useWallet } from '@solana/wallet-adapter-react';
import { useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { GameService } from '../services/GameService';
import { GameState } from '../types/game';

const GameContainer = styled.div`
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
  font-family: Arial, sans-serif;
`;

const CardGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 10px;
  margin: 20px 0;
`;

const Card = styled.button<{ selected: boolean }>`
  padding: 20px;
  font-size: 24px;
  border: 2px solid ${props => props.selected ? '#4CAF50' : '#ddd'};
  border-radius: 8px;
  background: ${props => props.selected ? '#E8F5E9' : '#fff'};
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: ${props => props.selected ? '#C8E6C9' : '#f5f5f5'};
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const PredictionGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 10px;
  margin: 20px 0;
`;

const Prediction = styled.button<{ selected: boolean }>`
  padding: 15px;
  font-size: 18px;
  border: 2px solid ${props => props.selected ? '#2196F3' : '#ddd'};
  border-radius: 8px;
  background: ${props => props.selected ? '#E3F2FD' : '#fff'};
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: ${props => props.selected ? '#BBDEFB' : '#f5f5f5'};
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const Button = styled.button`
  padding: 10px 20px;
  font-size: 16px;
  background: #2196F3;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  margin: 10px 0;

  &:hover {
    background: #1976D2;
  }

  &:disabled {
    background: #BDBDBD;
    cursor: not-allowed;
  }
`;

const GameStatus = styled.div`
  margin: 20px 0;
  padding: 15px;
  border-radius: 4px;
  background: #f5f5f5;
`;

const OpponentArea = styled.div`
  margin-top: 30px;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
`;

const Game: React.FC = () => {
  const wallet = useWallet();
  const { connection } = useConnection();
  const [gameService, setGameService] = useState<GameService | null>(null);
  const [selectedCard, setSelectedCard] = useState<number | null>(null);
  const [selectedPrediction, setSelectedPrediction] = useState<number | null>(null);
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [gameId, setGameId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [unsubscribe, setUnsubscribe] = useState<(() => void) | null>(null);

  useEffect(() => {
    if (wallet && connection) {
      try {
        // Using a valid Solana public key for testing
        const programId = new PublicKey('11111111111111111111111111111111');
        setGameService(new GameService({
          programId,
          connection,
          wallet
        }));
      } catch (err) {
        setError('Failed to initialize game service');
        console.error('Error initializing game service:', err);
      }
    }
  }, [wallet, connection]);

  useEffect(() => {
    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [unsubscribe]);

  const handleCardSelect = (card: number) => {
    setSelectedCard(card);
  };

  const handlePredictionSelect = (prediction: number) => {
    setSelectedPrediction(prediction);
  };

  const handleCreateGame = async () => {
    if (!gameService) return;
    try {
      const id = await gameService.createGame();
      setGameId(id);
      const unsubscribeFn = gameService.subscribeToGameState(id, setGameState);
      setUnsubscribe(() => unsubscribeFn);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error creating game');
    }
  };

  const handleJoinGame = async () => {
    if (!gameService || !gameId) return;
    try {
      await gameService.joinGame(gameId);
      const unsubscribeFn = gameService.subscribeToGameState(gameId, setGameState);
      setUnsubscribe(() => unsubscribeFn);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error joining game');
    }
  };

  const handleSubmitMove = async () => {
    if (!gameService || !gameId || !selectedCard || !selectedPrediction) return;
    try {
      const commitment = {
        card: selectedCard,
        prediction: selectedPrediction
      };
      await gameService.submitMove(gameId, commitment);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error submitting move');
    }
  };

  const handleRevealMove = async () => {
    if (!gameId || !selectedCard || !selectedPrediction || !gameService) return;
    try {
      await gameService.revealMove(gameId, selectedCard, selectedPrediction);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error revealing move');
    }
  };

  if (!gameService) {
    return <div>Loading game service...</div>;
  }

  return (
    <GameContainer>
      <h1>MORRA Game</h1>
      
      <div>
        <h2>Select Your Card (1-5)</h2>
        <CardGrid>
          {[1, 2, 3, 4, 5].map((card) => (
            <Card
              key={card}
              selected={selectedCard === card}
              onClick={() => handleCardSelect(card)}
              disabled={gameState?.status !== 'waiting' && gameState?.status !== 'in_progress'}
            >
              {card}
            </Card>
          ))}
        </CardGrid>
      </div>

      <div>
        <h2>Select Your Prediction (1-10)</h2>
        <PredictionGrid>
          {[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((prediction) => (
            <Prediction
              key={prediction}
              selected={selectedPrediction === prediction}
              onClick={() => handlePredictionSelect(prediction)}
              disabled={gameState?.status !== 'waiting' && gameState?.status !== 'in_progress'}
            >
              {prediction}
            </Prediction>
          ))}
        </PredictionGrid>
      </div>

      {!gameId ? (
        <Button onClick={handleCreateGame}>Create Game</Button>
      ) : (
        <>
          {!gameState?.opponent ? (
            <Button onClick={handleJoinGame}>Join Game</Button>
          ) : (
            <>
              {!gameState.creatorCommitment && !gameState.opponentCommitment ? (
                <Button onClick={handleSubmitMove}>Submit Move</Button>
              ) : (
                <Button onClick={handleRevealMove}>Reveal Move</Button>
              )}
            </>
          )}
        </>
      )}

      {error && <div style={{ color: 'red', margin: '10px 0' }}>{error}</div>}

      {gameState && (
        <GameStatus>
          <h3>Game Status</h3>
          <p>Status: {gameState.status}</p>
          <p>Creator: {gameState.creator}</p>
          <p>Opponent: {gameState.opponent || 'Waiting...'}</p>
          {gameState.winner && <p>Winner: {gameState.winner}</p>}
        </GameStatus>
      )}

      <OpponentArea>
        <h3>Opponent's Area</h3>
        {gameState?.opponentCard && gameState?.opponentPrediction ? (
          <>
            <p>Card: {gameState.opponentCard}</p>
            <p>Prediction: {gameState.opponentPrediction}</p>
          </>
        ) : (
          <p>Waiting for opponent's move...</p>
        )}
      </OpponentArea>
    </GameContainer>
  );
};

export default Game; 