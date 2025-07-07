use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    native_token::lamports_to_sol,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use anyhow::Result;
use log::{info, error};

pub async fn transfer_sol_manager(
    rpc_url: &str,
    sender_keypair: &Keypair,
    recipient_pubkey: &Pubkey,
    lamports: u64,
) -> Result<solana_sdk::signature::Signature> {
    info!(
        "[transfer_sol] Attempting to transfer {} lamports ({} SOL) from {} to {}",
        lamports,
        lamports_to_sol(lamports),
        sender_keypair.pubkey(),
        recipient_pubkey
    );

    let rpc_client = RpcClient::new(String::from(rpc_url));

    let instruction = system_instruction::transfer(
        &sender_keypair.pubkey(),
        recipient_pubkey,
        lamports,
    );

    let latest_blockhash = rpc_client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender_keypair.pubkey()),
        &[sender_keypair],
        latest_blockhash,
    );

    match rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(&tx, CommitmentConfig::confirmed()) {
        Ok(signature) => {
            info!(
                "[transfer_sol] Successfully transferred {} SOL to {}. Signature: {}",
                lamports_to_sol(lamports),
                recipient_pubkey,
                signature
            );
            Ok(signature)
        }
        Err(e) => {
            error!("[transfer_sol] SOL transfer failed: {}", e);
            Err(anyhow::anyhow!("SOL transfer failed: {}", e))
        }
    }
}
