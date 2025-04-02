import React from 'react';
import Game from './components/Game';
import WalletButton from './components/WalletButton';
import { WalletContextProvider } from './contexts/WalletContext';
import { createGlobalStyle } from 'styled-components';

const GlobalStyle = createGlobalStyle`
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  body {
    font-family: Arial, sans-serif;
    background-color: #1a1a1a;
    color: #ffffff;
  }

  .wallet-adapter-button {
    background-color: #4CAF50 !important;
    color: white !important;
    border: none !important;
    padding: 10px 20px !important;
    border-radius: 5px !important;
    cursor: pointer !important;
    transition: background-color 0.2s !important;
  }

  .wallet-adapter-button:hover {
    background-color: #45a049 !important;
  }

  .wallet-adapter-modal-wrapper {
    background-color: #2a2a2a !important;
  }

  .wallet-adapter-modal-button-close {
    background-color: #4CAF50 !important;
  }
`;

function App() {
  return (
    <WalletContextProvider>
      <GlobalStyle />
      <WalletButton />
      <Game />
    </WalletContextProvider>
  );
}

export default App;
