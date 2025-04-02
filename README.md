# Morra Game

A multiplayer implementation of the classic Morra game, built with React, TypeScript, and Vite. The game allows two players to play against each other using a simple card and prediction system.

## Features

- Real-time multiplayer gameplay
- Card selection (1-5)
- Prediction selection (1-10)
- Game state management
- Winner determination
- Modern UI with styled-components

## Getting Started

### Prerequisites

- Node.js (v14 or higher)
- Yarn package manager

### Installation

1. Clone the repository:
```bash
git clone https://github.com/StuartGrossman/Morra.git
cd Morra
```

2. Install dependencies:
```bash
cd client
yarn install
```

3. Start the development server:
```bash
yarn dev
```

4. Open your browser and navigate to `http://localhost:5173` (or the port shown in your terminal)

## How to Play

1. Create a game by clicking the "Create Game" button
2. Share the game ID with your opponent
3. Have your opponent join the game
4. Select a card (1-5) and a prediction (1-10)
5. Submit your move
6. Reveal your move when both players are ready
7. The winner is determined based on the total of both cards and the predictions

## Project Structure

```
client/
├── src/
│   ├── components/
│   │   └── Game.tsx
│   ├── services/
│   │   └── GameService.ts
│   ├── types/
│   │   └── game.ts
│   └── App.tsx
├── package.json
└── vite.config.ts
```

## Technologies Used

- React
- TypeScript
- Vite
- Styled Components
- Firebase (coming soon)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 