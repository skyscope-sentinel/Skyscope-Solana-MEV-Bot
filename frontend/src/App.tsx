import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import styled, { ThemeProvider, createGlobalStyle } from 'styled-components';
import { Line } from 'react-chartjs-2';
import QRCode from 'react-qr-code';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
);

// Theme definition
const theme = {
  colors: {
    background: '#121212',
    surface: '#1E1E1E',
    surfaceLight: '#2D2D2D',
    primary: '#BB86FC',
    primaryVariant: '#3700B3',
    secondary: '#03DAC6',
    error: '#CF6679',
    onBackground: '#FFFFFF',
    onSurface: '#FFFFFF',
    onPrimary: '#000000',
    onSecondary: '#000000',
    onError: '#000000',
    success: '#4CAF50',
    warning: '#FFC107',
    info: '#2196F3',
  },
  spacing: {
    xs: '4px',
    sm: '8px',
    md: '16px',
    lg: '24px',
    xl: '32px',
  },
  borderRadius: {
    sm: '4px',
    md: '8px',
    lg: '16px',
    xl: '24px',
    circle: '50%',
  },
  shadows: {
    sm: '0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24)',
    md: '0 3px 6px rgba(0,0,0,0.16), 0 3px 6px rgba(0,0,0,0.23)',
    lg: '0 10px 20px rgba(0,0,0,0.19), 0 6px 6px rgba(0,0,0,0.23)',
  },
  transitions: {
    default: 'all 0.3s cubic-bezier(.25,.8,.25,1)',
  },
  fontSizes: {
    xs: '12px',
    sm: '14px',
    md: '16px',
    lg: '20px',
    xl: '24px',
    xxl: '32px',
  },
};

// Global styles
const GlobalStyle = createGlobalStyle`
  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
      'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
      sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    background-color: ${({ theme }) => theme.colors.background};
    color: ${({ theme }) => theme.colors.onBackground};
    font-size: ${({ theme }) => theme.fontSizes.md};
    line-height: 1.5;
    overflow-x: hidden;
  }

  code, pre {
    font-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New', monospace;
    background-color: ${({ theme }) => theme.colors.surfaceLight};
    border-radius: ${({ theme }) => theme.borderRadius.sm};
    padding: ${({ theme }) => theme.spacing.xs};
  }

  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  ::-webkit-scrollbar-track {
    background: ${({ theme }) => theme.colors.surface};
  }

  ::-webkit-scrollbar-thumb {
    background: ${({ theme }) => theme.colors.primaryVariant};
    border-radius: ${({ theme }) => theme.borderRadius.md};
  }

  ::-webkit-scrollbar-thumb:hover {
    background: ${({ theme }) => theme.colors.primary};
  }
`;

// Styled Components
const AppContainer = styled.div`
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
`;

const Header = styled.header`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: ${({ theme }) => theme.spacing.md};
  background-color: ${({ theme }) => theme.colors.surface};
  border-bottom: 1px solid ${({ theme }) => theme.colors.surfaceLight};
`;

const Logo = styled.div`
  display: flex;
  align-items: center;
  
  h1 {
    font-size: ${({ theme }) => theme.fontSizes.lg};
    margin-left: ${({ theme }) => theme.spacing.md};
  }
`;

const MainContent = styled.main`
  flex: 1;
  display: flex;
  overflow: hidden;
`;

const Sidebar = styled.aside`
  width: 250px;
  background-color: ${({ theme }) => theme.colors.surface};
  padding: ${({ theme }) => theme.spacing.md};
  display: flex;
  flex-direction: column;
  border-right: 1px solid ${({ theme }) => theme.colors.surfaceLight};
`;

const Content = styled.section`
  flex: 1;
  padding: ${({ theme }) => theme.spacing.md};
  overflow-y: auto;
`;

const Card = styled.div`
  background-color: ${({ theme }) => theme.colors.surface};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => theme.spacing.md};
  margin-bottom: ${({ theme }) => theme.spacing.md};
  box-shadow: ${({ theme }) => theme.shadows.sm};
`;

const Button = styled.button<{ variant?: 'primary' | 'secondary' | 'error' | 'success' | 'warning' | 'info' }>`
  background-color: ${({ theme, variant }) => 
    variant === 'secondary' ? theme.colors.secondary :
    variant === 'error' ? theme.colors.error :
    variant === 'success' ? theme.colors.success :
    variant === 'warning' ? theme.colors.warning :
    variant === 'info' ? theme.colors.info :
    theme.colors.primary};
  color: ${({ theme, variant }) => 
    variant === 'secondary' ? theme.colors.onSecondary :
    variant === 'error' ? theme.colors.onError :
    variant === 'success' ? theme.colors.onBackground :
    variant === 'warning' ? theme.colors.onBackground :
    variant === 'info' ? theme.colors.onBackground :
    theme.colors.onPrimary};
  border: none;
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => `${theme.spacing.sm} ${theme.spacing.md}`};
  font-size: ${({ theme }) => theme.fontSizes.md};
  cursor: pointer;
  transition: ${({ theme }) => theme.transitions.default};
  display: inline-flex;
  align-items: center;
  justify-content: center;
  
  &:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }
  
  &:active {
    transform: translateY(1px);
  }
  
  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const Input = styled.input`
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  color: ${({ theme }) => theme.colors.onSurface};
  border: 1px solid ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => `${theme.spacing.sm} ${theme.spacing.md}`};
  font-size: ${({ theme }) => theme.fontSizes.md};
  width: 100%;
  transition: ${({ theme }) => theme.transitions.default};
  
  &:focus {
    outline: none;
    border-color: ${({ theme }) => theme.colors.primary};
  }
`;

const PinInput = styled(Input)`
  text-align: center;
  letter-spacing: 8px;
  font-size: ${({ theme }) => theme.fontSizes.lg};
  font-weight: bold;
  width: 160px;
`;

const FormGroup = styled.div`
  margin-bottom: ${({ theme }) => theme.spacing.md};
`;

const Label = styled.label`
  display: block;
  margin-bottom: ${({ theme }) => theme.spacing.xs};
  color: ${({ theme }) => theme.colors.onBackground};
  font-size: ${({ theme }) => theme.fontSizes.sm};
`;

const Flex = styled.div<{ direction?: 'row' | 'column', justify?: string, align?: string, gap?: string }>`
  display: flex;
  flex-direction: ${({ direction }) => direction || 'row'};
  justify-content: ${({ justify }) => justify || 'flex-start'};
  align-items: ${({ align }) => align || 'stretch'};
  gap: ${({ gap, theme }) => gap || theme.spacing.md};
`;

const NavItem = styled.div<{ active?: boolean }>`
  padding: ${({ theme }) => theme.spacing.md};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  cursor: pointer;
  transition: ${({ theme }) => theme.transitions.default};
  background-color: ${({ theme, active }) => active ? theme.colors.primaryVariant : 'transparent'};
  
  &:hover {
    background-color: ${({ theme, active }) => active ? theme.colors.primaryVariant : theme.colors.surfaceLight};
  }
`;

const ProgressBarContainer = styled.div`
  width: 100%;
  height: 8px;
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  overflow: hidden;
  margin: ${({ theme }) => `${theme.spacing.md} 0`};
`;

const ProgressBarFill = styled.div<{ progress: number }>`
  height: 100%;
  width: ${({ progress }) => `${progress}%`};
  background-color: ${({ theme }) => theme.colors.primary};
  transition: width 0.3s ease;
`;

const Badge = styled.span<{ variant?: 'primary' | 'secondary' | 'error' | 'success' | 'warning' | 'info' }>`
  background-color: ${({ theme, variant }) => 
    variant === 'secondary' ? theme.colors.secondary :
    variant === 'error' ? theme.colors.error :
    variant === 'success' ? theme.colors.success :
    variant === 'warning' ? theme.colors.warning :
    variant === 'info' ? theme.colors.info :
    theme.colors.primary};
  color: ${({ theme, variant }) => 
    variant === 'secondary' ? theme.colors.onSecondary :
    variant === 'error' ? theme.colors.onError :
    variant === 'success' ? theme.colors.onBackground :
    variant === 'warning' ? theme.colors.onBackground :
    variant === 'info' ? theme.colors.onBackground :
    theme.colors.onPrimary};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => `${theme.spacing.xs} ${theme.spacing.sm}`};
  font-size: ${({ theme }) => theme.fontSizes.xs};
  font-weight: bold;
  display: inline-block;
`;

const Stat = styled.div`
  display: flex;
  flex-direction: column;
  padding: ${({ theme }) => theme.spacing.md};
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  
  .label {
    font-size: ${({ theme }) => theme.fontSizes.sm};
    color: ${({ theme }) => theme.colors.onBackground};
    opacity: 0.7;
  }
  
  .value {
    font-size: ${({ theme }) => theme.fontSizes.lg};
    font-weight: bold;
  }
  
  .subvalue {
    font-size: ${({ theme }) => theme.fontSizes.xs};
    opacity: 0.7;
  }
`;

const CodeBlock = styled.pre`
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => theme.spacing.md};
  overflow-x: auto;
  font-family: 'Courier New', Courier, monospace;
  font-size: ${({ theme }) => theme.fontSizes.sm};
  margin: ${({ theme }) => theme.spacing.md} 0;
`;

const Modal = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
`;

const ModalContent = styled.div`
  background-color: ${({ theme }) => theme.colors.surface};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => theme.spacing.lg};
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: ${({ theme }) => theme.shadows.lg};
`;

const ModalHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: ${({ theme }) => theme.spacing.md};
  
  h2 {
    margin: 0;
  }
`;

const CloseButton = styled.button`
  background: none;
  border: none;
  color: ${({ theme }) => theme.colors.onBackground};
  font-size: ${({ theme }) => theme.fontSizes.xl};
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: ${({ theme }) => theme.borderRadius.circle};
  
  &:hover {
    background-color: ${({ theme }) => theme.colors.surfaceLight};
  }
`;

// Mock data for SOL to USD conversion (in a real app, this would come from an API or be stored locally)
const SOL_TO_USD = 150.75; // Example price

// App component
const App: React.FC = () => {
  // Authentication states
  const [isPinSetup, setIsPinSetup] = useState<boolean | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [pin, setPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [pinError, setPinError] = useState('');
  
  // Navigation state
  const [activeTab, setActiveTab] = useState('dashboard');
  
  // Wallet states
  const [wallets, setWallets] = useState<any[]>([]);
  const [selectedWallet, setSelectedWallet] = useState<string | null>(null);
  const [isImportingWallet, setIsImportingWallet] = useState(false);
  const [importMethod, setImportMethod] = useState<'file' | 'seed' | null>(null);
  const [seedPhrase, setSeedPhrase] = useState('');
  const [walletName, setWalletName] = useState('');
  
  // Trading states
  const [isTrading, setIsTrading] = useState(false);
  const [tradingProgress, setTradingProgress] = useState(0);
  const [solBalance, setSolBalance] = useState(0.05); // Starting with 0.05 SOL
  const [profitSol, setProfitSol] = useState(0);
  const [tradesExecuted, setTradesExecuted] = useState(0);
  const [tradingLogs, setTradingLogs] = useState<string[]>([]);
  
  // Chart data
  const [chartData, setChartData] = useState({
    labels: Array.from({ length: 10 }, (_, i) => (i + 1).toString()),
    datasets: [
      {
        label: 'Balance (SOL)',
        data: [0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05],
        borderColor: theme.colors.primary,
        backgroundColor: 'rgba(187, 134, 252, 0.1)',
        tension: 0.4,
      },
    ],
  });
  
  // Check if PIN is set up on component mount
  useEffect(() => {
    const checkPinSetup = async () => {
      try {
        const result = await invoke('is_pin_setup');
        setIsPinSetup(result as boolean);
      } catch (error) {
        console.error('Failed to check PIN setup:', error);
        setIsPinSetup(false);
      }
    };
    
    checkPinSetup();
  }, []);
  
  // Load wallets after authentication
  useEffect(() => {
    if (isAuthenticated) {
      loadWallets();
    }
  }, [isAuthenticated]);
  
  // Simulated trading effect
  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (isTrading) {
      interval = setInterval(() => {
        // Simulate trade execution
        const randomProfit = (Math.random() * 0.002) - 0.0005; // Random profit between -0.0005 and 0.0015 SOL
        const newProfit = profitSol + randomProfit;
        const newBalance = solBalance + randomProfit;
        
        setProfitSol(newProfit);
        setSolBalance(newBalance);
        setTradesExecuted(prev => prev + 1);
        setTradingProgress(Math.random() * 100);
        
        // Add log entry
        const profitText = randomProfit >= 0 ? `+${randomProfit.toFixed(6)}` : `${randomProfit.toFixed(6)}`;
        const logEntry = `[${new Date().toLocaleTimeString()}] Executed trade: ${profitText} SOL`;
        setTradingLogs(prev => [logEntry, ...prev].slice(0, 100));
        
        // Update chart data
        setChartData(prev => {
          const newLabels = [...prev.labels.slice(1), (parseInt(prev.labels[prev.labels.length - 1]) + 1).toString()];
          const newData = [...prev.datasets[0].data.slice(1), newBalance];
          
          return {
            labels: newLabels,
            datasets: [
              {
                ...prev.datasets[0],
                data: newData,
              },
            ],
          };
        });
      }, 3000);
    }
    
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [isTrading, profitSol, solBalance]);
  
  // Handle PIN setup
  const handlePinSetup = async () => {
    if (pin.length !== 4 || !/^\d+$/.test(pin)) {
      setPinError('PIN must be exactly 4 digits');
      return;
    }
    
    if (pin !== confirmPin) {
      setPinError('PINs do not match');
      return;
    }
    
    try {
      await invoke('setup_pin', { pin });
      setIsPinSetup(true);
      setIsAuthenticated(true);
      setPinError('');
    } catch (error) {
      console.error('Failed to set up PIN:', error);
      setPinError('Failed to set up PIN');
    }
  };
  
  // Handle PIN verification
  const handlePinVerification = async () => {
    if (pin.length !== 4 || !/^\d+$/.test(pin)) {
      setPinError('PIN must be exactly 4 digits');
      return;
    }
    
    try {
      const result = await invoke('verify_pin', { pin });
      if (result) {
        setIsAuthenticated(true);
        setPinError('');
      } else {
        setPinError('Invalid PIN');
      }
    } catch (error) {
      console.error('Failed to verify PIN:', error);
      setPinError('Failed to verify PIN');
    }
  };
  
  // Load wallets
  const loadWallets = async () => {
    try {
      const result = await invoke('list_wallets');
      setWallets(result as any[]);
      
      if ((result as any[]).length > 0) {
        setSelectedWallet((result as any[])[0].name);
      }
    } catch (error) {
      console.error('Failed to load wallets:', error);
    }
  };
  
  // Import wallet
  const handleWalletImport = async () => {
    if (!walletName) {
      alert('Please enter a wallet name');
      return;
    }
    
    try {
      let result;
      
      if (importMethod === 'file') {
        // In a real app, this would open a file picker
        alert('File import would open a file picker here');
        return;
      } else if (importMethod === 'seed') {
        if (!seedPhrase) {
          alert('Please enter a seed phrase');
          return;
        }
        
        result = await invoke('import_from_seed_phrase', { 
          name: walletName,
          seedPhrase,
          passphrase: '',
          pin
        });
      }
      
      setIsImportingWallet(false);
      setImportMethod(null);
      setSeedPhrase('');
      setWalletName('');
      
      // Reload wallets
      loadWallets();
    } catch (error) {
      console.error('Failed to import wallet:', error);
      alert(`Failed to import wallet: ${error}`);
    }
  };
  
  // Start trading
  const startTrading = async () => {
    if (!selectedWallet) {
      alert('Please select a wallet');
      return;
    }
    
    try {
      await invoke('start_trading', { wallet: selectedWallet });
      setIsTrading(true);
    } catch (error) {
      console.error('Failed to start trading:', error);
      alert(`Failed to start trading: ${error}`);
    }
  };
  
  // Stop trading
  const stopTrading = async () => {
    try {
      await invoke('stop_trading');
      setIsTrading(false);
    } catch (error) {
      console.error('Failed to stop trading:', error);
      alert(`Failed to stop trading: ${error}`);
    }
  };
  
  // Render authentication screen
  const renderAuthScreen = () => {
    if (isPinSetup === null) {
      return (
        <Card>
          <h2>Loading...</h2>
          <p>Checking PIN setup status...</p>
        </Card>
      );
    }
    
    if (!isPinSetup) {
      return (
        <Card>
          <h2>Welcome to Skyscope Solana MEV Bot</h2>
          <p>Please set up a 4-digit PIN to secure your wallet and trading operations.</p>
          
          <FormGroup>
            <Label htmlFor="pin">Enter a 4-digit PIN</Label>
            <PinInput
              id="pin"
              type="password"
              maxLength={4}
              value={pin}
              onChange={(e) => setPin(e.target.value)}
              placeholder="****"
            />
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="confirmPin">Confirm PIN</Label>
            <PinInput
              id="confirmPin"
              type="password"
              maxLength={4}
              value={confirmPin}
              onChange={(e) => setConfirmPin(e.target.value)}
              placeholder="****"
            />
          </FormGroup>
          
          {pinError && <p style={{ color: theme.colors.error }}>{pinError}</p>}
          
          <Button onClick={handlePinSetup}>Set PIN</Button>
        </Card>
      );
    }
    
    return (
      <Card>
        <h2>Welcome Back</h2>
        <p>Enter your PIN to continue.</p>
        
        <FormGroup>
          <Label htmlFor="pin">Enter your 4-digit PIN</Label>
          <PinInput
            id="pin"
            type="password"
            maxLength={4}
            value={pin}
            onChange={(e) => setPin(e.target.value)}
            placeholder="****"
          />
        </FormGroup>
        
        {pinError && <p style={{ color: theme.colors.error }}>{pinError}</p>}
        
        <Button onClick={handlePinVerification}>Unlock</Button>
      </Card>
    );
  };
  
  // Render wallet import modal
  const renderWalletImportModal = () => {
    if (!isImportingWallet) return null;
    
    return (
      <Modal>
        <ModalContent>
          <ModalHeader>
            <h2>Import Wallet</h2>
            <CloseButton onClick={() => setIsImportingWallet(false)}>×</CloseButton>
          </ModalHeader>
          
          <FormGroup>
            <Label htmlFor="walletName">Wallet Name</Label>
            <Input
              id="walletName"
              type="text"
              value={walletName}
              onChange={(e) => setWalletName(e.target.value)}
              placeholder="My Wallet"
            />
          </FormGroup>
          
          <FormGroup>
            <Label>Import Method</Label>
            <Flex>
              <Button
                variant={importMethod === 'file' ? 'primary' : 'secondary'}
                onClick={() => setImportMethod('file')}
              >
                Keypair File
              </Button>
              <Button
                variant={importMethod === 'seed' ? 'primary' : 'secondary'}
                onClick={() => setImportMethod('seed')}
              >
                Seed Phrase
              </Button>
            </Flex>
          </FormGroup>
          
          {importMethod === 'file' && (
            <FormGroup>
              <Label htmlFor="keypairFile">Keypair File</Label>
              <Input
                id="keypairFile"
                type="file"
                accept=".json"
              />
              <p style={{ fontSize: theme.fontSizes.sm, opacity: 0.7, marginTop: theme.spacing.xs }}>
                Select your Solflare or other Solana wallet JSON file.
              </p>
            </FormGroup>
          )}
          
          {importMethod === 'seed' && (
            <FormGroup>
              <Label htmlFor="seedPhrase">Seed Phrase</Label>
              <Input
                id="seedPhrase"
                as="textarea"
                rows={3}
                value={seedPhrase}
                onChange={(e) => setSeedPhrase(e.target.value)}
                placeholder="Enter your 12 or 24 word seed phrase"
                style={{ resize: 'vertical' }}
              />
              <p style={{ fontSize: theme.fontSizes.sm, opacity: 0.7, marginTop: theme.spacing.xs }}>
                Enter your 12 or 24 word seed phrase separated by spaces.
              </p>
            </FormGroup>
          )}
          
          <Flex justify="flex-end" gap={theme.spacing.sm}>
            <Button variant="error" onClick={() => setIsImportingWallet(false)}>Cancel</Button>
            <Button onClick={handleWalletImport}>Import</Button>
          </Flex>
        </ModalContent>
      </Modal>
    );
  };
  
  // Render wallet management
  const renderWalletManagement = () => {
    return (
      <>
        <h2>Wallet Management</h2>
        
        <Flex justify="space-between" align="center" style={{ marginBottom: theme.spacing.md }}>
          <h3>Your Wallets</h3>
          <Button onClick={() => setIsImportingWallet(true)}>Import Wallet</Button>
        </Flex>
        
        {wallets.length === 0 ? (
          <Card>
            <p>No wallets found. Import a wallet to get started.</p>
            <Button onClick={() => setIsImportingWallet(true)}>Import Wallet</Button>
          </Card>
        ) : (
          <div>
            {wallets.map((wallet, index) => (
              <Card key={index} style={{ 
                borderLeft: selectedWallet === wallet.name ? `4px solid ${theme.colors.primary}` : 'none',
                cursor: 'pointer'
              }} onClick={() => setSelectedWallet(wallet.name)}>
                <Flex justify="space-between" align="center">
                  <div>
                    <h3>{wallet.name}</h3>
                    <p style={{ fontSize: theme.fontSizes.sm, opacity: 0.7 }}>{wallet.pubkey}</p>
                  </div>
                  <Badge variant={selectedWallet === wallet.name ? 'primary' : 'secondary'}>
                    {selectedWallet === wallet.name ? 'Selected' : 'Select'}
                  </Badge>
                </Flex>
              </Card>
            ))}
          </div>
        )}
        
        {renderWalletImportModal()}
      </>
    );
  };
  
  // Render dashboard
  const renderDashboard = () => {
    return (
      <>
        <h2>Trading Dashboard</h2>
        
        <Flex gap={theme.spacing.md} style={{ marginBottom: theme.spacing.md }}>
          <Stat style={{ flex: 1 }}>
            <span className="label">Current Balance</span>
            <span className="value">{solBalance.toFixed(6)} SOL</span>
            <span className="subvalue">${(solBalance * SOL_TO_USD).toFixed(2)} USD</span>
          </Stat>
          
          <Stat style={{ flex: 1 }}>
            <span className="label">Profit/Loss</span>
            <span className="value" style={{ color: profitSol >= 0 ? theme.colors.success : theme.colors.error }}>
              {profitSol >= 0 ? '+' : ''}{profitSol.toFixed(6)} SOL
            </span>
            <span className="subvalue">${(profitSol * SOL_TO_USD).toFixed(2)} USD</span>
          </Stat>
          
          <Stat style={{ flex: 1 }}>
            <span className="label">Trades Executed</span>
            <span className="value">{tradesExecuted}</span>
            <span className="subvalue">Since start</span>
          </Stat>
        </Flex>
        
        <Card>
          <Flex justify="space-between" align="center" style={{ marginBottom: theme.spacing.md }}>
            <h3>Trading Activity</h3>
            {isTrading ? (
              <Button variant="error" onClick={stopTrading}>Stop Trading</Button>
            ) : (
              <Button variant="success" onClick={startTrading}>Start Trading</Button>
            )}
          </Flex>
          
          <div style={{ height: '300px' }}>
            <Line 
              data={chartData}
              options={{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: {
                    position: 'top',
                  },
                  title: {
                    display: true,
                    text: 'Balance History',
                  },
                },
                scales: {
                  y: {
                    beginAtZero: false,
                  },
                },
              }}
            />
          </div>
          
          {isTrading && (
            <>
              <h4 style={{ marginTop: theme.spacing.md }}>Current Operation</h4>
              <ProgressBarContainer>
                <ProgressBarFill progress={tradingProgress} />
              </ProgressBarContainer>
              <p>Scanning for opportunities...</p>
            </>
          )}
        </Card>
        
        <Card>
          <h3>Trading Logs</h3>
          <CodeBlock>
            {tradingLogs.length === 0 ? (
              'No trading activity yet. Start trading to see logs.'
            ) : (
              tradingLogs.map((log, index) => (
                <div key={index}>{log}</div>
              ))
            )}
          </CodeBlock>
        </Card>
      </>
    );
  };
  
  // Render settings
  const renderSettings = () => {
    return (
      <>
        <h2>Settings</h2>
        
        <Card>
          <h3>Trading Parameters</h3>
          
          <FormGroup>
            <Label htmlFor="tradingAmount">Trading Amount (SOL)</Label>
            <Input
              id="tradingAmount"
              type="number"
              min="0.05"
              step="0.01"
              defaultValue="0.1"
            />
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="maxSlippage">Max Slippage (%)</Label>
            <Input
              id="maxSlippage"
              type="number"
              min="0.1"
              max="5.0"
              step="0.1"
              defaultValue="1.0"
            />
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="strategy">Trading Strategy</Label>
            <select
              id="strategy"
              style={{
                backgroundColor: theme.colors.surfaceLight,
                color: theme.colors.onSurface,
                border: `1px solid ${theme.colors.surfaceLight}`,
                borderRadius: theme.borderRadius.md,
                padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                fontSize: theme.fontSizes.md,
                width: '100%',
              }}
            >
              <option value="0">MEV Arbitrage</option>
              <option value="1">Sandwich Trading</option>
              <option value="2">Flashloan Arbitrage</option>
              <option value="3">Liquidity Sniping</option>
            </select>
          </FormGroup>
          
          <Button>Save Settings</Button>
        </Card>
        
        <Card>
          <h3>Security</h3>
          
          <Button>Change PIN</Button>
        </Card>
      </>
    );
  };
  
  // Render help
  const renderHelp = () => {
    return (
      <>
        <h2>Help & Guide</h2>
        
        <Card>
          <h3>Getting Started</h3>
          <p>Welcome to the Skyscope Solana MEV Bot! This guide will help you get started with the application.</p>
          
          <h4 style={{ marginTop: theme.spacing.md }}>Step 1: Import Your Wallet</h4>
          <p>Start by importing your Solflare wallet or any other Solana wallet. You can do this from the Wallet tab.</p>
          
          <h4 style={{ marginTop: theme.spacing.md }}>Step 2: Configure Trading Settings</h4>
          <p>Go to the Settings tab to configure your trading parameters, such as the amount to trade, maximum slippage, and trading strategy.</p>
          
          <h4 style={{ marginTop: theme.spacing.md }}>Step 3: Start Trading</h4>
          <p>Once your wallet is imported and settings are configured, go to the Dashboard tab and click "Start Trading" to begin.</p>
          
          <h4 style={{ marginTop: theme.spacing.md }}>Step 4: Monitor Your Profits</h4>
          <p>The Dashboard will show you real-time information about your trading activity, including your current balance, profit/loss, and trading logs.</p>
        </Card>
        
        <Card>
          <h3>Frequently Asked Questions</h3>
          
          <h4 style={{ marginTop: theme.spacing.md }}>What is MEV?</h4>
          <p>MEV (Maximal Extractable Value) refers to the maximum value that can be extracted from block production in excess of the standard block reward and gas fees by including, excluding, or reordering transactions in a block.</p>
          
          <h4 style={{ marginTop: theme.spacing.md }}>How does the bot make money?</h4>
          <p>The bot uses various strategies such as arbitrage between different DEXs, sandwich trading, and flashloan arbitrage to capture value from the Solana blockchain.</p>
          
          <h4 style={{ marginTop: theme.spacing.md }}>Is my seed phrase safe?</h4>
          <p>Yes, your seed phrase is never stored on disk. It is only used once to derive your keypair, which is then encrypted with your PIN and stored securely.</p>
        </Card>
      </>
    );
  };
  
  // Render main content based on active tab
  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return renderDashboard();
      case 'wallet':
        return renderWalletManagement();
      case 'settings':
        return renderSettings();
      case 'help':
        return renderHelp();
      default:
        return renderDashboard();
    }
  };
  
  return (
    <ThemeProvider theme={theme}>
      <GlobalStyle />
      <AppContainer>
        {!isAuthenticated ? (
          <Flex direction="column" justify="center" align="center" style={{ height: '100vh' }}>
            {renderAuthScreen()}
          </Flex>
        ) : (
          <>
            <Header>
              <Logo>
                <h1>Skyscope Solana MEV Bot</h1>
              </Logo>
              <Flex>
                <Button variant="secondary" onClick={() => setIsAuthenticated(false)}>Logout</Button>
              </Flex>
            </Header>
            
            <MainContent>
              <Sidebar>
                <NavItem active={activeTab === 'dashboard'} onClick={() => setActiveTab('dashboard')}>
                  Dashboard
                </NavItem>
                <NavItem active={activeTab === 'wallet'} onClick={() => setActiveTab('wallet')}>
                  Wallet
                </NavItem>
                <NavItem active={activeTab === 'settings'} onClick={() => setActiveTab('settings')}>
                  Settings
                </NavItem>
                <NavItem active={activeTab === 'help'} onClick={() => setActiveTab('help')}>
                  Help & Guide
                </NavItem>
                
                {selectedWallet && (
                  <Card style={{ marginTop: 'auto' }}>
                    <h4>Selected Wallet</h4>
                    <p>{selectedWallet}</p>
                    {isTrading ? (
                      <Badge variant="success">Trading Active</Badge>
                    ) : (
                      <Badge variant="warning">Trading Inactive</Badge>
                    )}
                  </Card>
                )}
              </Sidebar>
              
              <Content>
                {renderContent()}
              </Content>
            </MainContent>
          </>
        )}
      </AppContainer>
    </ThemeProvider>
  );
};

export default App;
