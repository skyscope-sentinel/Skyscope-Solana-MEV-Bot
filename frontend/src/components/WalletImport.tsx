import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { readTextFile } from '@tauri-apps/api/fs';
import styled from 'styled-components';

// Styled components
const WizardContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: ${({ theme }) => theme.spacing.md};
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
`;

const WizardHeader = styled.div`
  text-align: center;
  margin-bottom: ${({ theme }) => theme.spacing.lg};
  
  h2 {
    font-size: ${({ theme }) => theme.fontSizes.xl};
    margin-bottom: ${({ theme }) => theme.spacing.sm};
    background: linear-gradient(45deg, ${({ theme }) => theme.colors.primary}, ${({ theme }) => theme.colors.secondary});
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  p {
    color: ${({ theme }) => theme.colors.onBackground};
    opacity: 0.8;
  }
`;

const StepIndicator = styled.div`
  display: flex;
  justify-content: space-between;
  margin-bottom: ${({ theme }) => theme.spacing.lg};
  position: relative;
  
  &::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 0;
    right: 0;
    height: 2px;
    background-color: ${({ theme }) => theme.colors.surfaceLight};
    transform: translateY(-50%);
    z-index: 0;
  }
`;

const Step = styled.div<{ active: boolean; completed: boolean }>`
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: ${({ theme, active, completed }) => 
    completed ? theme.colors.success : 
    active ? theme.colors.primary : 
    theme.colors.surfaceLight};
  color: ${({ theme, active, completed }) => 
    completed || active ? theme.colors.onPrimary : 
    theme.colors.onSurface};
  font-weight: bold;
  z-index: 1;
  transition: all 0.3s ease;
  position: relative;
  
  &::after {
    content: '${({ active, completed }) => completed ? '✓' : ''}';
    position: absolute;
  }
`;

const StepLabel = styled.div<{ active: boolean }>`
  position: absolute;
  top: 45px;
  transform: translateX(-50%);
  font-size: ${({ theme }) => theme.fontSizes.xs};
  color: ${({ theme, active }) => active ? theme.colors.primary : theme.colors.onBackground};
  opacity: ${({ active }) => active ? 1 : 0.6};
  white-space: nowrap;
  text-align: center;
  width: 100px;
`;

const Card = styled.div`
  background-color: ${({ theme }) => theme.colors.surface};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => theme.spacing.lg};
  box-shadow: ${({ theme }) => theme.shadows.md};
  margin-bottom: ${({ theme }) => theme.spacing.md};
`;

const MethodSelector = styled.div`
  display: flex;
  gap: ${({ theme }) => theme.spacing.md};
  margin-bottom: ${({ theme }) => theme.spacing.md};
`;

const MethodButton = styled.button<{ selected: boolean }>`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: ${({ theme }) => theme.spacing.lg};
  background-color: ${({ theme, selected }) => selected ? theme.colors.primary + '33' : theme.colors.surfaceLight};
  border: 2px solid ${({ theme, selected }) => selected ? theme.colors.primary : 'transparent'};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  cursor: pointer;
  transition: all 0.2s ease;
  
  &:hover {
    background-color: ${({ theme, selected }) => selected ? theme.colors.primary + '33' : theme.colors.surfaceLight + '99'};
  }
  
  .icon {
    font-size: 2rem;
    margin-bottom: ${({ theme }) => theme.spacing.sm};
    color: ${({ theme, selected }) => selected ? theme.colors.primary : theme.colors.onBackground};
  }
  
  .label {
    font-weight: ${({ selected }) => selected ? 'bold' : 'normal'};
    color: ${({ theme, selected }) => selected ? theme.colors.primary : theme.colors.onBackground};
  }
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

const TextArea = styled.textarea`
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  color: ${({ theme }) => theme.colors.onSurface};
  border: 1px solid ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => `${theme.spacing.sm} ${theme.spacing.md}`};
  font-size: ${({ theme }) => theme.fontSizes.md};
  width: 100%;
  min-height: 120px;
  resize: vertical;
  transition: ${({ theme }) => theme.transitions.default};
  
  &:focus {
    outline: none;
    border-color: ${({ theme }) => theme.colors.primary};
  }
`;

const FileInput = styled.div`
  display: flex;
  flex-direction: column;
  gap: ${({ theme }) => theme.spacing.sm};
`;

const FilePlaceholder = styled.div`
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  border: 2px dashed ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => theme.spacing.lg};
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  
  &:hover {
    border-color: ${({ theme }) => theme.colors.primary};
  }
  
  .icon {
    font-size: 2rem;
    margin-bottom: ${({ theme }) => theme.spacing.sm};
    color: ${({ theme }) => theme.colors.onBackground};
    opacity: 0.6;
  }
  
  .text {
    text-align: center;
  }
`;

const SelectedFile = styled.div`
  background-color: ${({ theme }) => theme.colors.surfaceLight};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => theme.spacing.md};
  display: flex;
  align-items: center;
  justify-content: space-between;
  
  .file-info {
    display: flex;
    align-items: center;
    gap: ${({ theme }) => theme.spacing.sm};
  }
  
  .file-icon {
    color: ${({ theme }) => theme.colors.primary};
  }
  
  .file-name {
    font-weight: bold;
  }
  
  .remove-button {
    color: ${({ theme }) => theme.colors.error};
    background: none;
    border: none;
    cursor: pointer;
    padding: ${({ theme }) => theme.spacing.xs};
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    
    &:hover {
      background-color: ${({ theme }) => theme.colors.error + '33'};
    }
  }
`;

const SeedWordContainer = styled.div`
  display: flex;
  flex-wrap: wrap;
  gap: ${({ theme }) => theme.spacing.xs};
  margin-top: ${({ theme }) => theme.spacing.sm};
`;

const SeedWord = styled.div<{ isValid: boolean }>`
  background-color: ${({ theme, isValid }) => isValid ? theme.colors.surfaceLight : theme.colors.error + '33'};
  border-radius: ${({ theme }) => theme.borderRadius.sm};
  padding: ${({ theme }) => `${theme.spacing.xs} ${theme.spacing.sm}`};
  font-size: ${({ theme }) => theme.fontSizes.sm};
  display: inline-block;
`;

const HelpText = styled.p`
  font-size: ${({ theme }) => theme.fontSizes.sm};
  color: ${({ theme }) => theme.colors.onBackground};
  opacity: 0.7;
  margin-top: ${({ theme }) => theme.spacing.xs};
`;

const ErrorText = styled.p`
  font-size: ${({ theme }) => theme.fontSizes.sm};
  color: ${({ theme }) => theme.colors.error};
  margin-top: ${({ theme }) => theme.spacing.xs};
`;

const ButtonContainer = styled.div`
  display: flex;
  justify-content: space-between;
  margin-top: ${({ theme }) => theme.spacing.lg};
`;

const Button = styled.button<{ variant?: 'primary' | 'secondary' | 'error' }>`
  background-color: ${({ theme, variant }) => 
    variant === 'secondary' ? theme.colors.surfaceLight : 
    variant === 'error' ? theme.colors.error : 
    theme.colors.primary};
  color: ${({ theme, variant }) => 
    variant === 'secondary' ? theme.colors.onSurface : 
    variant === 'error' ? theme.colors.onError : 
    theme.colors.onPrimary};
  border: none;
  border-radius: ${({ theme }) => theme.borderRadius.md};
  padding: ${({ theme }) => `${theme.spacing.sm} ${theme.spacing.lg}`};
  font-size: ${({ theme }) => theme.fontSizes.md};
  font-weight: bold;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: ${({ theme }) => theme.spacing.sm};
  
  &:hover {
    opacity: 0.9;
  }
  
  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const SuccessAnimation = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: ${({ theme }) => theme.spacing.xl};
  
  .check-circle {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background-color: ${({ theme }) => theme.colors.success};
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: ${({ theme }) => theme.spacing.lg};
    animation: scale-in 0.5s ease-out;
  }
  
  .check-mark {
    color: white;
    font-size: 3rem;
  }
  
  h3 {
    margin-bottom: ${({ theme }) => theme.spacing.md};
  }
  
  @keyframes scale-in {
    0% {
      transform: scale(0);
    }
    70% {
      transform: scale(1.1);
    }
    100% {
      transform: scale(1);
    }
  }
`;

const InfoBox = styled.div`
  background-color: ${({ theme }) => theme.colors.info + '33'};
  border-left: 4px solid ${({ theme }) => theme.colors.info};
  padding: ${({ theme }) => theme.spacing.md};
  border-radius: ${({ theme }) => theme.borderRadius.sm};
  margin-bottom: ${({ theme }) => theme.spacing.md};
  
  .title {
    color: ${({ theme }) => theme.colors.info};
    font-weight: bold;
    margin-bottom: ${({ theme }) => theme.spacing.xs};
  }
  
  .content {
    font-size: ${({ theme }) => theme.fontSizes.sm};
  }
`;

// Types
interface WalletImportProps {
  onComplete: (walletInfo: { name: string; pubkey: string }) => void;
  onCancel: () => void;
  pin: string;
}

// Main component
const WalletImport: React.FC<WalletImportProps> = ({ onComplete, onCancel, pin }) => {
  // State
  const [currentStep, setCurrentStep] = useState(1);
  const [importMethod, setImportMethod] = useState<'file' | 'seed' | null>(null);
  const [walletName, setWalletName] = useState('');
  const [seedPhrase, setSeedPhrase] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [selectedFile, setSelectedFile] = useState<{ path: string; name: string } | null>(null);
  const [seedWords, setSeedWords] = useState<{ word: string; isValid: boolean }[]>([]);
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isComplete, setIsComplete] = useState(false);
  const [importedWallet, setImportedWallet] = useState<{ name: string; pubkey: string } | null>(null);

  // BIP39 word list (simplified version - in a real app, we'd use a complete list)
  const bip39WordList = [
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd", "abuse",
    "access", "accident", "account", "accuse", "achieve", "acid", "acoustic", "acquire", "across", "act",
    "action", "actor", "actress", "actual", "adapt", "add", "addict", "address", "adjust", "admit",
    "adult", "advance", "advice", "aerobic", "affair", "afford", "afraid", "again", "age", "agent",
    "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album", "alcohol", "alert",
    "alien", "all", "alley", "allow", "almost", "alone", "alpha", "already", "also", "alter",
    "always", "amateur", "amazing", "among", "amount", "amused", "analyst", "anchor", "ancient", "anger",
    "angle", "angry", "animal", "ankle", "announce", "annual", "another", "answer", "antenna", "antique",
    "anxiety", "any", "apart", "apology", "appear", "apple", "approve", "april", "arch", "arctic",
    "area", "arena", "argue", "arm", "armed", "armor", "army", "around", "arrange", "arrest",
    "arrive", "arrow", "art", "artefact", "artist", "artwork", "ask", "aspect", "assault", "asset",
    "assist", "assume", "asthma", "athlete", "atom", "attack", "attend", "attitude", "attract", "auction",
    "audit", "august", "aunt", "author", "auto", "autumn", "average", "avocado", "avoid", "awake",
    "aware", "away", "awesome", "awful", "awkward", "axis", "baby", "bachelor", "bacon", "badge",
    "bag", "balance", "balcony", "ball", "bamboo", "banana", "banner", "bar", "barely", "bargain",
    "barrel", "base", "basic", "basket", "battle", "beach", "bean", "beauty", "because", "become",
    "beef", "before", "begin", "behave", "behind", "believe", "below", "belt", "bench", "benefit"
    // ... and many more words in a real implementation
  ];

  // Effect to validate seed phrase
  useEffect(() => {
    if (seedPhrase) {
      const words = seedPhrase.trim().toLowerCase().split(/\s+/);
      const validatedWords = words.map(word => ({
        word,
        isValid: bip39WordList.includes(word)
      }));
      setSeedWords(validatedWords);
    } else {
      setSeedWords([]);
    }
  }, [seedPhrase]);

  // Handle file selection
  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Solana Keypair', extensions: ['json'] }]
      });
      
      if (selected && typeof selected === 'string') {
        // Get just the filename from the path
        const name = selected.split('/').pop() || selected.split('\\').pop() || selected;
        
        setSelectedFile({
          path: selected,
          name
        });
        
        // Auto-generate wallet name from filename (without extension)
        if (!walletName) {
          const nameWithoutExt = name.replace(/\.json$/, '');
          setWalletName(nameWithoutExt);
        }
        
        setError('');
      }
    } catch (err) {
      console.error('Error selecting file:', err);
      setError('Failed to select file. Please try again.');
    }
  };

  // Handle next step
  const handleNextStep = () => {
    // Validate current step
    if (currentStep === 1) {
      if (!importMethod) {
        setError('Please select an import method');
        return;
      }
      setError('');
    } else if (currentStep === 2) {
      if (!walletName.trim()) {
        setError('Please enter a wallet name');
        return;
      }
      setError('');
    } else if (currentStep === 3) {
      if (importMethod === 'file' && !selectedFile) {
        setError('Please select a keypair file');
        return;
      } else if (importMethod === 'seed') {
        const words = seedWords.length;
        if (words !== 12 && words !== 24) {
          setError('Seed phrase must be 12 or 24 words');
          return;
        }
        
        const invalidWords = seedWords.filter(w => !w.isValid);
        if (invalidWords.length > 0) {
          setError(`Invalid seed words detected: ${invalidWords.map(w => w.word).join(', ')}`);
          return;
        }
      }
      setError('');
    }
    
    // Move to next step
    setCurrentStep(prev => prev + 1);
  };

  // Handle previous step
  const handlePrevStep = () => {
    setCurrentStep(prev => prev - 1);
    setError('');
  };

  // Handle import
  const handleImport = async () => {
    setIsLoading(true);
    setError('');
    
    try {
      let result;
      
      if (importMethod === 'file' && selectedFile) {
        // Read file content
        const fileContent = await readTextFile(selectedFile.path);
        
        // Import from file
        result = await invoke('import_from_file', {
          name: walletName,
          filePath: selectedFile.path,
          pin
        });
      } else if (importMethod === 'seed') {
        // Import from seed phrase
        result = await invoke('import_from_seed_phrase', {
          name: walletName,
          seedPhrase,
          passphrase,
          pin
        });
      }
      
      // Set imported wallet info
      if (result) {
        setImportedWallet(result as { name: string; pubkey: string });
        setIsComplete(true);
      }
    } catch (err) {
      console.error('Error importing wallet:', err);
      setError(`Failed to import wallet: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Handle completion
  const handleComplete = () => {
    if (importedWallet) {
      onComplete(importedWallet);
    }
  };

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 1:
        return (
          <Card>
            <h3>Select Import Method</h3>
            <p>Choose how you want to import your Solana wallet:</p>
            
            <MethodSelector>
              <MethodButton 
                selected={importMethod === 'file'} 
                onClick={() => setImportMethod('file')}
              >
                <div className="icon">📄</div>
                <div className="label">Keypair File</div>
                <div style={{ fontSize: '0.8rem', opacity: 0.7 }}>Solflare, Phantom</div>
              </MethodButton>
              
              <MethodButton 
                selected={importMethod === 'seed'} 
                onClick={() => setImportMethod('seed')}
              >
                <div className="icon">🔑</div>
                <div className="label">Seed Phrase</div>
                <div style={{ fontSize: '0.8rem', opacity: 0.7 }}>12 or 24 words</div>
              </MethodButton>
            </MethodSelector>
            
            {importMethod === 'file' && (
              <InfoBox>
                <div className="title">About Keypair Files</div>
                <div className="content">
                  <p>A keypair file contains your wallet's private key in encrypted form. You can export this file from wallets like Solflare or Phantom.</p>
                  <p>To export from Solflare: Settings → Security → Export Keypair</p>
                </div>
              </InfoBox>
            )}
            
            {importMethod === 'seed' && (
              <InfoBox>
                <div className="title">About Seed Phrases</div>
                <div className="content">
                  <p>A seed phrase (also called recovery phrase or mnemonic) is a list of 12 or 24 words that can recreate your wallet.</p>
                  <p>⚠️ Never share your seed phrase with anyone or enter it on untrusted websites!</p>
                </div>
              </InfoBox>
            )}
          </Card>
        );
      
      case 2:
        return (
          <Card>
            <h3>Name Your Wallet</h3>
            <p>Choose a name for this wallet to easily identify it in the app:</p>
            
            <FormGroup>
              <Label htmlFor="walletName">Wallet Name</Label>
              <Input
                id="walletName"
                type="text"
                value={walletName}
                onChange={(e) => setWalletName(e.target.value)}
                placeholder="e.g., My Trading Wallet"
                autoFocus
              />
              <HelpText>Choose a memorable name like "Trading Wallet" or "Arbitrage Bot"</HelpText>
            </FormGroup>
          </Card>
        );
      
      case 3:
        return (
          <Card>
            <h3>{importMethod === 'file' ? 'Select Keypair File' : 'Enter Seed Phrase'}</h3>
            
            {importMethod === 'file' && (
              <FileInput>
                {!selectedFile ? (
                  <FilePlaceholder onClick={handleFileSelect}>
                    <div className="icon">📁</div>
                    <div className="text">
                      <p>Click to select your Solana keypair file</p>
                      <p style={{ fontSize: '0.8rem', opacity: 0.7 }}>Supports .json files exported from Solflare or Phantom</p>
                    </div>
                  </FilePlaceholder>
                ) : (
                  <SelectedFile>
                    <div className="file-info">
                      <div className="file-icon">📄</div>
                      <div className="file-name">{selectedFile.name}</div>
                    </div>
                    <button 
                      className="remove-button" 
                      onClick={() => setSelectedFile(null)}
                      aria-label="Remove file"
                    >
                      ✕
                    </button>
                  </SelectedFile>
                )}
                <Button variant="secondary" onClick={handleFileSelect}>
                  Browse Files
                </Button>
              </FileInput>
            )}
            
            {importMethod === 'seed' && (
              <>
                <FormGroup>
                  <Label htmlFor="seedPhrase">Seed Phrase (12 or 24 words)</Label>
                  <TextArea
                    id="seedPhrase"
                    value={seedPhrase}
                    onChange={(e) => setSeedPhrase(e.target.value)}
                    placeholder="Enter your seed phrase words separated by spaces"
                    autoFocus
                  />
                  <HelpText>Enter your 12 or 24 words separated by spaces</HelpText>
                  
                  {seedWords.length > 0 && (
                    <SeedWordContainer>
                      {seedWords.map((word, index) => (
                        <SeedWord key={index} isValid={word.isValid}>
                          {word.word}
                        </SeedWord>
                      ))}
                    </SeedWordContainer>
                  )}
                </FormGroup>
                
                <FormGroup>
                  <Label htmlFor="passphrase">Passphrase (Optional)</Label>
                  <Input
                    id="passphrase"
                    type="password"
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                    placeholder="Enter passphrase if you have one"
                  />
                  <HelpText>Only enter if you created your wallet with an additional passphrase</HelpText>
                </FormGroup>
              </>
            )}
          </Card>
        );
      
      case 4:
        return (
          <Card>
            <h3>Confirm Import</h3>
            <p>Please review your wallet import details:</p>
            
            <div style={{ margin: '20px 0' }}>
              <p><strong>Import Method:</strong> {importMethod === 'file' ? 'Keypair File' : 'Seed Phrase'}</p>
              <p><strong>Wallet Name:</strong> {walletName}</p>
              {importMethod === 'file' && selectedFile && (
                <p><strong>File:</strong> {selectedFile.name}</p>
              )}
              {importMethod === 'seed' && (
                <p><strong>Seed Phrase:</strong> {seedWords.length} words {passphrase ? '(with passphrase)' : ''}</p>
              )}
            </div>
            
            <InfoBox>
              <div className="title">Security Note</div>
              <div className="content">
                <p>Your wallet will be encrypted with your PIN and stored securely on your device.</p>
                <p>If you're importing with a seed phrase, it will only be used once and never stored.</p>
              </div>
            </InfoBox>
          </Card>
        );
      
      case 5:
        if (isComplete && importedWallet) {
          return (
            <Card>
              <SuccessAnimation>
                <div className="check-circle">
                  <div className="check-mark">✓</div>
                </div>
                <h3>Wallet Imported Successfully!</h3>
                <p>Your wallet has been securely imported and encrypted.</p>
                <p style={{ marginTop: '10px' }}><strong>Wallet Name:</strong> {importedWallet.name}</p>
                <p><strong>Public Key:</strong> {importedWallet.pubkey}</p>
                <Button onClick={handleComplete} style={{ marginTop: '20px' }}>
                  Start Trading
                </Button>
              </SuccessAnimation>
            </Card>
          );
        }
        
        return (
          <Card>
            <h3>Importing Wallet</h3>
            <p>Please wait while we import and encrypt your wallet...</p>
            
            <div style={{ margin: '30px 0', textAlign: 'center' }}>
              {/* Simple loading animation */}
              <div style={{ 
                display: 'inline-block',
                width: '50px',
                height: '50px',
                border: '5px solid rgba(187, 134, 252, 0.3)',
                borderRadius: '50%',
                borderTop: '5px solid #BB86FC',
                animation: 'spin 1s linear infinite'
              }} />
              <style>
                {`
                  @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                  }
                `}
              </style>
            </div>
          </Card>
        );
      
      default:
        return null;
    }
  };

  // Render step indicators
  const renderStepIndicators = () => {
    const steps = [
      { number: 1, label: 'Method' },
      { number: 2, label: 'Name' },
      { number: 3, label: 'Import' },
      { number: 4, label: 'Confirm' },
      { number: 5, label: 'Complete' }
    ];
    
    return (
      <StepIndicator>
        {steps.map((step) => (
          <div key={step.number} style={{ position: 'relative' }}>
            <Step 
              active={currentStep === step.number} 
              completed={currentStep > step.number}
            >
              {currentStep > step.number ? '' : step.number}
            </Step>
            <StepLabel active={currentStep === step.number}>
              {step.label}
            </StepLabel>
          </div>
        ))}
      </StepIndicator>
    );
  };

  // Render buttons
  const renderButtons = () => {
    // On the last step, we either show the loading state or no buttons (success)
    if (currentStep === 5) {
      if (!isComplete) {
        return null; // No buttons during loading
      }
      return null; // No buttons after success (we have a button in the success content)
    }
    
    return (
      <ButtonContainer>
        {currentStep > 1 ? (
          <Button variant="secondary" onClick={handlePrevStep}>
            Back
          </Button>
        ) : (
          <Button variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
        )}
        
        {currentStep === 4 ? (
          <Button onClick={handleImport} disabled={isLoading}>
            {isLoading ? 'Importing...' : 'Import Wallet'}
          </Button>
        ) : (
          <Button onClick={handleNextStep}>
            Next
          </Button>
        )}
      </ButtonContainer>
    );
  };

  return (
    <WizardContainer>
      <WizardHeader>
        <h2>Import Your Solana Wallet</h2>
        <p>Follow these simple steps to securely import your wallet</p>
      </WizardHeader>
      
      {renderStepIndicators()}
      
      {renderStepContent()}
      
      {error && <ErrorText>{error}</ErrorText>}
      
      {renderButtons()}
    </WizardContainer>
  );
};

export default WalletImport;
