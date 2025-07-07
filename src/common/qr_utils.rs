use qrcodegen::{QrCode, QrCodeEcc};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;
use anyhow::Result;
use log::info;
use crate::common::constants::Env; // For Env type
use crate::common::budget_manager; // For get_sol_balance
use crate::common::utils as common_utils; // For get_sol_usdt_price

/// Generates a QR code for the given text data and prints it to the console.
/// The QR code is printed as a series of block characters.
fn print_text_qr_code(text: &str) -> Result<()> { // Renamed to avoid conflict if we make a general print_qr_code public
    let err_cor_lvl: QrCodeEcc = QrCodeEcc::Low; // Error correction level
    let qr = QrCode::encode_text(text, err_cor_lvl)?;
    let border = 2; // Smaller border for console
    for y in -border..qr.size() + border {
        let mut line = String::new();
        for x in -border..qr.size() + border {
            line.push(if qr.get_module(x, y) { '█' } else { ' ' });
            // line.push(' '); // Removing extra space for denser QR
        }
        println!("{}", line);
    }
    Ok(())
}

/// Displays funding information for a bot instance, including its public key and a QR code.
pub async fn display_funding_info(
    env: &Env,
    instance_id: usize,
    keypair_path: &str,
    suggested_budget_usdt: f64
) -> Result<()> {
    info!("----------------------------------------------------------------------");
    info!("Funding Information for Bot Instance ID: {}", instance_id);
    info!("----------------------------------------------------------------------");

    match read_keypair_file(keypair_path) {
        Ok(keypair) => {
            let pubkey = keypair.pubkey();
            info!("Wallet Public Key: {}", pubkey.to_string());

            match budget_manager::get_sol_balance(&env.rpc_url, &pubkey).await {
                Ok(balance) => {
                    match common_utils::get_sol_usdt_price().await {
                        Ok(price) if price > 0.0 => {
                            info!("Current Balance:   {:.9} SOL (~${:.2} USDT)", balance, balance * price);
                        }
                        _ => {
                            info!("Current Balance:   {:.9} SOL (USDT price unavailable)", balance);
                        }
                    }
                }
                Err(e) => {
                    info!("Could not fetch current balance: {}", e);
                }
            }

            info!("Suggested minimum funding for budget: {:.2} USDT worth of SOL.", suggested_budget_usdt);
            info!("Additionally, ensure a small amount for transaction fees (e.g., 0.01-0.05 SOL).");
            info!("Scan the QR code below with your Solana wallet app to send SOL to this address:");

            if let Err(e) = print_text_qr_code(&pubkey.to_string()) {
                info!("(Could not print QR code: {})", e);
                info!("Please copy the public key above directly.");
            }
        }
        Err(e) => {
            info!("Error loading keypair from path {}: {}", keypair_path, e);
            info!("Cannot display funding information for this instance.");
        }
    }
    info!("----------------------------------------------------------------------\n");
    Ok(())
}
