import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { dialog } from '@tauri-apps/api';
import './WalletImport.css';

type ImportMethod = 'solflare' | 'seedphrase' | 'keypair';
type Step = 'method' | 'solflare' | 'seedphrase' | 'keypair' | 'pin' | 'complete';

interface WalletImportProps {
  onComplete: (success: boolean) => void;
}

const WalletImport: React.FC<WalletImportProps> = ({ onComplete }) => {
  // State management
  const [currentStep, setCurrentStep] = useState<Step>('method');
  const [importMethod, setImportMethod] = useState<ImportMethod>('solflare');
  const [seedPhrase, setSeedPhrase] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [solflareFile, setSolflareFile] = useState<string | null>(null);
  const [solflarePassword, setSolflarePassword] = useState('');
  const [pin, setPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [walletName, setWalletName] = useState('My Wallet');

  // Step titles and descriptions
  const stepInfo = {
    method: {
      title: 'Choose Import Method',
      description: 'Select how you want to import your wallet'
    },
    solflare: {
      title: 'Import Solflare Wallet',
      description: 'Upload your Solflare JSON file and enter password'
    },
    seedphrase: {
      title: 'Import Seed Phrase',
      description: 'Enter your 12 or 24-word recovery phrase'
    },
    keypair: {
      title: 'Import Private Key',
      description: 'Enter your wallet private key'
    },
    pin: {
      title: 'Create PIN',
      description: 'Create a 4-digit PIN to secure your wallet'
    },
    complete: {
      title: 'Wallet Imported',
      description: 'Your wallet has been successfully imported'
    }
  };

  // Handle method selection
  const handleMethodSelect = (method: ImportMethod) => {
    setImportMethod(method);
    setCurrentStep(method);
    setError(null);
  };

  // Handle Solflare file selection
  const handleFileSelect = async () => {
    try {
      const selected = await dialog.open({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }],
        multiple: false
      });

      if (selected && !Array.isArray(selected)) {
        setSolflareFile(selected as string);
        setError(null);
      }
    } catch (err) {
      console.error('File selection error:', err);
      setError('Failed to select file. Please try again.');
    }
  };

  // Validate seed phrase
  const validateSeedPhrase = (phrase: string): boolean => {
    const words = phrase.trim().split(/\s+/);
    return words.length === 12 || words.length === 24;
  };

  // Validate private key
  const validatePrivateKey = (key: string): boolean => {
    // Basic validation - should be 64 hex characters
    return /^[0-9a-fA-F]{64}$/.test(key.trim());
  };

  // Validate PIN
  const validatePin = (): boolean => {
    return pin.length === 4 && pin === confirmPin;
  };

  // Handle next step
  const handleNext = () => {
    setError(null);

    // Validation based on current step
    if (currentStep === 'solflare') {
      if (!solflareFile) {
        setError('Please select a Solflare JSON file');
        return;
      }
      if (!solflarePassword) {
        setError('Please enter your Solflare wallet password');
        return;
      }
      setCurrentStep('pin');
    } 
    else if (currentStep === 'seedphrase') {
      if (!validateSeedPhrase(seedPhrase)) {
        setError('Please enter a valid 12 or 24-word seed phrase');
        return;
      }
      setCurrentStep('pin');
    } 
    else if (currentStep === 'keypair') {
      if (!validatePrivateKey(privateKey)) {
        setError('Please enter a valid private key (64 hex characters)');
        return;
      }
      setCurrentStep('pin');
    } 
    else if (currentStep === 'pin') {
      if (!validatePin()) {
        setError('Please enter a valid 4-digit PIN and ensure it matches the confirmation');
        return;
      }
      handleImport();
    }
  };

  // Handle back
  const handleBack = () => {
    if (currentStep === 'solflare' || currentStep === 'seedphrase' || currentStep === 'keypair') {
      setCurrentStep('method');
    } else if (currentStep === 'pin') {
      setCurrentStep(importMethod);
    }
    setError(null);
  };

  // Handle wallet import
  const handleImport = async () => {
    setLoading(true);
    setError(null);

    try {
      let result = false;

      if (importMethod === 'solflare') {
        result = await invoke('import_solflare_wallet', {
          filePath: solflareFile,
          password: solflarePassword,
          pin,
          name: walletName
        });
      } else if (importMethod === 'seedphrase') {
        result = await invoke('import_seed_phrase', {
          seedPhrase,
          pin,
          name: walletName
        });
      } else if (importMethod === 'keypair') {
        result = await invoke('import_private_key', {
          privateKey,
          pin,
          name: walletName
        });
      }

      if (result) {
        setCurrentStep('complete');
      } else {
        setError('Failed to import wallet. Please check your inputs and try again.');
      }
    } catch (err) {
      console.error('Wallet import error:', err);
      setError(`Import failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // Handle completion
  const handleComplete = () => {
    onComplete(currentStep === 'complete');
  };

  // Render method selection step
  const renderMethodStep = () => (
    <div className="method-selection">
      <div 
        className={`method-card ${importMethod === 'solflare' ? 'selected' : ''}`}
        onClick={() => handleMethodSelect('solflare')}
      >
        <div className="method-icon solflare-icon"></div>
        <h3>Solflare Wallet</h3>
        <p>Import using Solflare JSON file</p>
      </div>
      
      <div 
        className={`method-card ${importMethod === 'seedphrase' ? 'selected' : ''}`}
        onClick={() => handleMethodSelect('seedphrase')}
      >
        <div className="method-icon seed-icon"></div>
        <h3>Seed Phrase</h3>
        <p>Import using 12 or 24-word recovery phrase</p>
      </div>
      
      <div 
        className={`method-card ${importMethod === 'keypair' ? 'selected' : ''}`}
        onClick={() => handleMethodSelect('keypair')}
      >
        <div className="method-icon key-icon"></div>
        <h3>Private Key</h3>
        <p>Import using wallet private key</p>
      </div>
    </div>
  );

  // Render Solflare import step
  const renderSolflareStep = () => (
    <div className="solflare-import">
      <div className="file-upload">
        <button 
          className="file-select-button"
          onClick={handleFileSelect}
        >
          {solflareFile ? 'Change File' : 'Select Solflare JSON File'}
        </button>
        
        {solflareFile && (
          <div className="selected-file">
            Selected: {solflareFile.split('/').pop()}
          </div>
        )}
      </div>
      
      <div className="form-group">
        <label>Wallet Password</label>
        <input 
          type="password"
          value={solflarePassword}
          onChange={(e) => setSolflarePassword(e.target.value)}
          placeholder="Enter your Solflare wallet password"
        />
      </div>
      
      <div className="form-group">
        <label>Wallet Name (Optional)</label>
        <input 
          type="text"
          value={walletName}
          onChange={(e) => setWalletName(e.target.value)}
          placeholder="My Wallet"
        />
      </div>
    </div>
  );

  // Render seed phrase import step
  const renderSeedPhraseStep = () => (
    <div className="seedphrase-import">
      <div className="form-group">
        <label>Seed Phrase</label>
        <textarea
          value={seedPhrase}
          onChange={(e) => setSeedPhrase(e.target.value)}
          placeholder="Enter your 12 or 24-word recovery phrase separated by spaces"
          rows={4}
          className="seed-textarea"
        />
      </div>
      
      <div className="seed-info">
        <div className="info-icon">ℹ️</div>
        <p>
          Your seed phrase is never sent to any server and is only used locally to 
          generate your wallet. It will be encrypted and stored securely.
        </p>
      </div>
      
      <div className="form-group">
        <label>Wallet Name (Optional)</label>
        <input 
          type="text"
          value={walletName}
          onChange={(e) => setWalletName(e.target.value)}
          placeholder="My Wallet"
        />
      </div>
    </div>
  );

  // Render private key import step
  const renderKeypairStep = () => (
    <div className="keypair-import">
      <div className="form-group">
        <label>Private Key</label>
        <input 
          type="password"
          value={privateKey}
          onChange={(e) => setPrivateKey(e.target.value)}
          placeholder="Enter your private key (64 hex characters)"
        />
      </div>
      
      <div className="key-info">
        <div className="info-icon">ℹ️</div>
        <p>
          Your private key is never sent to any server and is only used locally.
          It will be encrypted and stored securely.
        </p>
      </div>
      
      <div className="form-group">
        <label>Wallet Name (Optional)</label>
        <input 
          type="text"
          value={walletName}
          onChange={(e) => setWalletName(e.target.value)}
          placeholder="My Wallet"
        />
      </div>
    </div>
  );

  // Render PIN creation step
  const renderPinStep = () => (
    <div className="pin-creation">
      <div className="form-group">
        <label>Create PIN (4 digits)</label>
        <input 
          type="password"
          maxLength={4}
          value={pin}
          onChange={(e) => setPin(e.target.value.replace(/[^0-9]/g, ''))}
          placeholder="Enter 4-digit PIN"
        />
      </div>
      
      <div className="form-group">
        <label>Confirm PIN</label>
        <input 
          type="password"
          maxLength={4}
          value={confirmPin}
          onChange={(e) => setConfirmPin(e.target.value.replace(/[^0-9]/g, ''))}
          placeholder="Confirm 4-digit PIN"
        />
      </div>
      
      <div className="pin-info">
        <div className="info-icon">ℹ️</div>
        <p>
          This PIN will be used to encrypt your wallet and will be required each time
          you open the application. Please remember it as it cannot be recovered.
        </p>
      </div>
    </div>
  );

  // Render completion step
  const renderCompleteStep = () => (
    <div className="import-complete">
      <div className="success-icon">✓</div>
      <h3>Wallet Successfully Imported!</h3>
      <p>You can now start trading with your wallet.</p>
    </div>
  );

  // Render current step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 'method':
        return renderMethodStep();
      case 'solflare':
        return renderSolflareStep();
      case 'seedphrase':
        return renderSeedPhraseStep();
      case 'keypair':
        return renderKeypairStep();
      case 'pin':
        return renderPinStep();
      case 'complete':
        return renderCompleteStep();
      default:
        return null;
    }
  };

  return (
    <div className="wallet-import-overlay">
      <div className="wallet-import-modal">
        <div className="modal-header">
          <h2>{stepInfo[currentStep].title}</h2>
          <button 
            className="close-button"
            onClick={() => onComplete(false)}
          >
            ✕
          </button>
        </div>
        
        <div className="modal-body">
          <p className="step-description">{stepInfo[currentStep].description}</p>
          
          {renderStepContent()}
          
          {error && (
            <div className="error-message">
              {error}
            </div>
          )}
        </div>
        
        <div className="modal-footer">
          {currentStep !== 'method' && currentStep !== 'complete' && (
            <button 
              className="back-button"
              onClick={handleBack}
              disabled={loading}
            >
              Back
            </button>
          )}
          
          {currentStep !== 'complete' ? (
            <button 
              className="next-button"
              onClick={currentStep === 'method' ? () => handleMethodSelect(importMethod) : handleNext}
              disabled={loading}
            >
              {currentStep === 'pin' ? 'Import Wallet' : 'Next'}
              {loading && <span className="loading-spinner"></span>}
            </button>
          ) : (
            <button 
              className="complete-button"
              onClick={handleComplete}
            >
              Start Trading
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default WalletImport;
