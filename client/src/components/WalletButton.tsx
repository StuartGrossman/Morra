import React from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import styled from 'styled-components';

const WalletButtonContainer = styled.div`
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 1000;
`;

const WalletButton: React.FC = () => {
  const { connected, publicKey } = useWallet();

  return (
    <WalletButtonContainer>
      <WalletMultiButton />
      {connected && (
        <div style={{ 
          color: '#4CAF50', 
          marginTop: '10px', 
          fontSize: '14px',
          textAlign: 'center' 
        }}>
          Connected: {publicKey?.toString().slice(0, 4)}...{publicKey?.toString().slice(-4)}
        </div>
      )}
    </WalletButtonContainer>
  );
};

export default WalletButton; 