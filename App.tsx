import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend } from 'chart.js';
import { Line } from 'react-chartjs-2';
import WalletImport from './components/WalletImport';
import './App.css';

// Register Chart.js components
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend);

// Mock data for development
const mockBalanceHistory = Array.from({ length: 24 }, (_, i) => ({
  time: `${i}:00`,
  balance: 10 + Math.random() * 2,
  profit: (Math.random() * 0.5).toFixed(4)
}));

function App() {
  // State management
  const [authenticated, setAuthenticated] = useState(false);
  const [pin, setPin] = useState('');
  const [walletConnected, setWalletConnected] = useState(false);
  const [walletBalance, setWalletBalance] = useState(0);
  const [usdBalance, setUsdBalance] = useState(0);
  const [solPrice, setSolPrice] = useState(150.25); // Mock SOL price in USD
  const [activeTab, setActiveTab] = useState('dashboard');
  const [isTrading, setIsTrading] = useState(false);
  const [tradingStrategy, setTradingStrategy] = useState('arbitrage');
  const [profitToday, setProfitToday] = useState(0);
  const [totalProfit, setTotalProfit] = useState(0);
  const [tradeCount, setTradeCount] = useState(0);
  const [successRate, setSuccessRate] = useState(0);
  const [showWalletImport, setShowWalletImport] = useState(false);
  const [notifications, setNotifications] = useState([]);
  const [balanceHistory, setBalanceHistory] = useState(mockBalanceHistory);
  const [currentProgress, setCurrentProgress] = useState(0);
  const [currentOperation, setCurrentOperation] = useState('');

  // Authentication effect
  useEffect(() => {
    const checkAuth = async () => {
      try {
        const isAuth = await invoke('check_authentication');
        setAuthenticated(isAuth);
        if (isAuth) {
          fetchWalletData();
        }
      } catch (error) {
        console.error('Authentication check failed:', error);
      }
    };
    
    checkAuth();
    
    // Mock data updates for demo purposes
    if (authenticated && walletConnected) {
      const interval = setInterval(() => {
        if (isTrading) {
          updateMockData();
        }
      }, 5000);
      
      return () => clearInterval(interval);
    }
  }, [authenticated, walletConnected, isTrading]);

  // Update mock trading data
  const updateMockData = () => {
    // Update balance with small fluctuations
    const newBalance = walletBalance + (Math.random() * 0.02 - 0.01);
    setWalletBalance(newBalance);
    setUsdBalance(newBalance * solPrice);
    
    // Update profit
    const newProfit = profitToday + (Math.random() * 0.005);
    setProfitToday(newProfit);
    setTotalProfit(totalProfit + (Math.random() * 0.002));
    
    // Update trade count occasionally
    if (Math.random() > 0.7) {
      setTradeCount(tradeCount + 1);
      setSuccessRate(Math.min(100, successRate + (Math.random() > 0.8 ? 1 : -0.5)));
      
      // Add notification
      const strategies = ['arbitrage', 'sandwich', 'flashloan'];
      const tokens = ['SOL/USDC', 'RAY/USDC', 'BONK/SOL', 'JUP/USDC'];
      const profitAmount = (Math.random() * 0.01).toFixed(4);
      
      setNotifications(prev => [
        {
          id: Date.now(),
          type: Math.random() > 0.2 ? 'success' : 'warning',
          message: `${Math.random() > 0.2 ? 'Successful' : 'Attempted'} ${strategies[Math.floor(Math.random() * strategies.length)]} trade on ${tokens[Math.floor(Math.random() * tokens.length)]} (${profitAmount} SOL)`
        },
        ...prev.slice(0, 9)
      ]);
    }
    
    // Update chart data
    const now = new Date();
    const timeStr = `${now.getHours()}:${now.getMinutes().toString().padStart(2, '0')}`;
    
    setBalanceHistory(prev => [
      ...prev.slice(1),
      {
        time: timeStr,
        balance: newBalance,
        profit: (Math.random() * 0.01).toFixed(4)
      }
    ]);
    
    // Update progress for current operation
    if (currentOperation) {
      const newProgress = currentProgress + Math.random() * 10;
      if (newProgress >= 100) {
        setCurrentProgress(0);
        setCurrentOperation('');
      } else {
        setCurrentProgress(newProgress);
      }
    } else if (Math.random() > 0.9) {
      // Occasionally start a new operation
      const operations = [
        'Scanning for arbitrage opportunities',
        'Analyzing price impact',
        'Preparing sandwich trade',
        'Optimizing trade route',
        'Calculating slippage'
      ];
      setCurrentOperation(operations[Math.floor(Math.random() * operations.length)]);
      setCurrentProgress(Math.random() * 30);
    }
  };

  // Fetch wallet data from backend
  const fetchWalletData = async () => {
    try {
      const walletData = await invoke('get_wallet_data');
      setWalletConnected(true);
      setWalletBalance(walletData.balance);
      setUsdBalance(walletData.balance * solPrice);
    } catch (error) {
      console.error('Failed to fetch wallet data:', error);
    }
  };

  // Handle PIN authentication
  const handleAuthenticate = async () => {
    if (pin.length !== 4) return;
    
    try {
      const result = await invoke('authenticate', { pin });
      if (result) {
        setAuthenticated(true);
        fetchWalletData();
      } else {
        alert('Invalid PIN. Please try again.');
      }
    } catch (error) {
      console.error('Authentication failed:', error);
      alert('Authentication failed. Please try again.');
    }
  };

  // Handle trading start/stop
  const toggleTrading = async () => {
    if (!walletConnected) {
      setShowWalletImport(true);
      return;
    }
    
    try {
      if (isTrading) {
        await invoke('stop_trading');
      } else {
        await invoke('start_trading', { strategy: tradingStrategy });
        // Start a new operation
        setCurrentOperation('Initializing trading engine');
        setCurrentProgress(5);
      }
      setIsTrading(!isTrading);
    } catch (error) {
      console.error('Failed to toggle trading:', error);
      alert(`Failed to ${isTrading ? 'stop' : 'start'} trading. Please try again.`);
    }
  };

  // Handle wallet import completion
  const handleWalletImported = (success) => {
    setShowWalletImport(false);
    if (success) {
      setWalletConnected(true);
      fetchWalletData();
    }
  };

  // Chart configuration
  const chartData = {
    labels: balanceHistory.map(item => item.time),
    datasets: [
      {
        label: 'Wallet Balance (SOL)',
        data: balanceHistory.map(item => item.balance),
        borderColor: 'rgba(75, 192, 192, 1)',
        backgroundColor: 'rgba(75, 192, 192, 0.2)',
        tension: 0.4
      }
    ]
  };

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'top',
        labels: {
          color: 'rgba(255, 255, 255, 0.7)'
        }
      },
      tooltip: {
        mode: 'index',
        intersect: false,
      }
    },
    scales: {
      y: {
        ticks: { color: 'rgba(255, 255, 255, 0.7)' },
        grid: { color: 'rgba(255, 255, 255, 0.1)' }
      },
      x: {
        ticks: { color: 'rgba(255, 255, 255, 0.7)' },
        grid: { color: 'rgba(255, 255, 255, 0.1)' }
      }
    }
  };

  // Render PIN authentication screen
  if (!authenticated) {
    return (
      <div className="auth-container">
        <div className="auth-card">
          <h1>Skyscope Solana MEV Bot</h1>
          <p>Enter your 4-digit PIN to unlock</p>
          <input
            type="password"
            maxLength={4}
            value={pin}
            onChange={(e) => setPin(e.target.value.replace(/[^0-9]/g, ''))}
            placeholder="PIN"
            className="pin-input"
          />
          <button 
            className="auth-button"
            onClick={handleAuthenticate}
            disabled={pin.length !== 4}
          >
            Unlock
          </button>
        </div>
      </div>
    );
  }

  // Render main application
  return (
    <div className="app-container">
      {/* Sidebar */}
      <div className="sidebar">
        <div className="logo">
          <h2>Skyscope</h2>
          <p>Solana MEV Bot</p>
        </div>
        
        <nav className="nav-menu">
          <button 
            className={`nav-item ${activeTab === 'dashboard' ? 'active' : ''}`}
            onClick={() => setActiveTab('dashboard')}
          >
            Dashboard
          </button>
          <button 
            className={`nav-item ${activeTab === 'wallet' ? 'active' : ''}`}
            onClick={() => setActiveTab('wallet')}
          >
            Wallet
          </button>
          <button 
            className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            Settings
          </button>
        </nav>
        
        <div className="sidebar-footer">
          <p>v1.0.0</p>
        </div>
      </div>
      
      {/* Main content */}
      <div className="main-content">
        {/* Header */}
        <header className="header">
          <h1>{activeTab.charAt(0).toUpperCase() + activeTab.slice(1)}</h1>
          
          <div className="wallet-status">
            {walletConnected ? (
              <>
                <div className="balance-display">
                  <span className="balance-amount">{walletBalance.toFixed(4)} SOL</span>
                  <span className="balance-usd">(${usdBalance.toFixed(2)})</span>
                </div>
                <div className="connection-status connected">
                  Connected
                </div>
              </>
            ) : (
              <button 
                className="connect-wallet-button"
                onClick={() => setShowWalletImport(true)}
              >
                Connect Wallet
              </button>
            )}
          </div>
        </header>
        
        {/* Dashboard content */}
        {activeTab === 'dashboard' && (
          <div className="dashboard">
            {/* Stats cards */}
            <div className="stats-container">
              <div className="stat-card">
                <h3>Profit Today</h3>
                <div className="stat-value">
                  <span>{profitToday.toFixed(4)} SOL</span>
                  <span className="stat-usd">${(profitToday * solPrice).toFixed(2)}</span>
                </div>
              </div>
              
              <div className="stat-card">
                <h3>Total Profit</h3>
                <div className="stat-value">
                  <span>{totalProfit.toFixed(4)} SOL</span>
                  <span className="stat-usd">${(totalProfit * solPrice).toFixed(2)}</span>
                </div>
              </div>
              
              <div className="stat-card">
                <h3>Trades</h3>
                <div className="stat-value">{tradeCount}</div>
              </div>
              
              <div className="stat-card">
                <h3>Success Rate</h3>
                <div className="stat-value">{successRate.toFixed(1)}%</div>
              </div>
            </div>
            
            {/* Chart */}
            <div className="chart-container">
              <h3>Balance History</h3>
              <div className="chart-wrapper">
                <Line data={chartData} options={chartOptions} />
              </div>
            </div>
            
            {/* Trading controls */}
            <div className="trading-controls">
              <div className="strategy-selector">
                <h3>Trading Strategy</h3>
                <select 
                  value={tradingStrategy}
                  onChange={(e) => setTradingStrategy(e.target.value)}
                  disabled={isTrading}
                >
                  <option value="arbitrage">Arbitrage</option>
                  <option value="sandwich">Sandwich Trading</option>
                  <option value="flashloan">Flashloan Arbitrage</option>
                  <option value="liquidity">Liquidity Sniping</option>
                </select>
              </div>
              
              <button 
                className={`trading-button ${isTrading ? 'stop' : 'start'}`}
                onClick={toggleTrading}
                disabled={!walletConnected && !isTrading}
              >
                {isTrading ? 'Stop Trading' : 'Start Trading'}
              </button>
            </div>
            
            {/* Current operation */}
            {currentOperation && (
              <div className="operation-status">
                <h3>{currentOperation}</h3>
                <div className="progress-bar-container">
                  <div 
                    className="progress-bar" 
                    style={{ width: `${currentProgress}%` }}
                  ></div>
                </div>
              </div>
            )}
            
            {/* Notifications */}
            <div className="notifications">
              <h3>Recent Activity</h3>
              {notifications.length === 0 ? (
                <p className="no-activity">No recent activity</p>
              ) : (
                <ul className="notification-list">
                  {notifications.map(notification => (
                    <li 
                      key={notification.id}
                      className={`notification-item ${notification.type}`}
                    >
                      {notification.message}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        )}
        
        {/* Wallet content */}
        {activeTab === 'wallet' && (
          <div className="wallet-page">
            <h2>Wallet Management</h2>
            
            {walletConnected ? (
              <div className="wallet-details">
                <div className="wallet-info-card">
                  <h3>Wallet Balance</h3>
                  <div className="wallet-balance">
                    <span className="sol-amount">{walletBalance.toFixed(4)} SOL</span>
                    <span className="usd-amount">${usdBalance.toFixed(2)}</span>
                  </div>
                </div>
                
                <button 
                  className="import-new-wallet"
                  onClick={() => setShowWalletImport(true)}
                >
                  Import Different Wallet
                </button>
              </div>
            ) : (
              <div className="wallet-connect-prompt">
                <p>Connect your wallet to start trading</p>
                <button 
                  className="connect-wallet-button large"
                  onClick={() => setShowWalletImport(true)}
                >
                  Connect Wallet
                </button>
              </div>
            )}
          </div>
        )}
        
        {/* Settings content */}
        {activeTab === 'settings' && (
          <div className="settings-page">
            <h2>Settings</h2>
            
            <div className="settings-section">
              <h3>Security</h3>
              <button className="settings-button">Change PIN</button>
            </div>
            
            <div className="settings-section">
              <h3>Trading Parameters</h3>
              
              <div className="setting-item">
                <label>Maximum Trade Size (SOL)</label>
                <input type="number" defaultValue={1} min={0.1} step={0.1} />
              </div>
              
              <div className="setting-item">
                <label>Maximum Slippage (%)</label>
                <input type="number" defaultValue={1} min={0.1} max={5} step={0.1} />
              </div>
              
              <div className="setting-item">
                <label>Auto Stop-Loss (%)</label>
                <input type="number" defaultValue={5} min={1} max={20} step={1} />
              </div>
            </div>
          </div>
        )}
      </div>
      
      {/* Wallet import modal */}
      {showWalletImport && (
        <WalletImport onComplete={handleWalletImported} />
      )}
    </div>
  );
}

export default App;
