use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    native_token::lamports_to_sol,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use anyhow::Result;
use log::{info, warn};

use crate::common::constants::Env;
use crate::common::utils::get_sol_usdt_price; // Assuming this utility exists

// Placeholder for loading a keypair from path, ideally use a shared utility if available
// For now, this is a simplified version. Real keypair loading is more complex.
fn load_keypair_from_path(path: &str) -> Result<Keypair> {
    // This is a highly simplified placeholder.
    // In a real scenario, you'd read the file and parse it into a Keypair.
    // e.g., using std::fs::read_to_string and then parsing the byte array.
    // For now, to make it compilable without actual file I/O here:
    warn!("Using placeholder keypair loading for path: {}. Implement actual file loading.", path);
    Ok(Keypair::new()) // Returns a new random keypair, NOT from path.
}


/// Fetches the current SOL balance for a given public key.
pub async fn get_sol_balance(rpc_url: &str, pubkey: &Pubkey) -> Result<f64> {
    let rpc_client = RpcClient::new(String::from(rpc_url));
    let lamports = rpc_client.get_balance(pubkey)?;
    Ok(lamports_to_sol(lamports))
}

/// Checks if the spending amount (in SOL) is within the budget (in USDT).
pub async fn is_within_budget(
    spending_sol: f64,
    budget_usdt: f64,
    current_sol_balance: f64, // Current balance of the account that will spend
    sol_price_usdt: Option<f64>, // Optional pre-fetched SOL price
) -> Result<bool> {
    let sol_price = match sol_price_usdt {
        Some(price) => price,
        None => get_sol_usdt_price().await.map_err(|e| anyhow::anyhow!("Failed to get SOL/USDT price for budget check: {}", e))?,
    };

    if sol_price <= 0.0 {
        return Err(anyhow::anyhow!("Invalid SOL price (<=0) for budget check."));
    }

    let budget_sol = budget_usdt / sol_price;
    info!("Budget Check: Max Budget: {:.6} SOL ({:.2} USDT). Proposed Spend: {:.6} SOL. Current Balance: {:.6} SOL", budget_sol, budget_usdt, spending_sol, current_sol_balance);

    if spending_sol > current_sol_balance {
        warn!("Budget Check: Proposed spend ({:.6} SOL) exceeds current balance ({:.6} SOL).", spending_sol, current_sol_balance);
        return Ok(false);
    }

    // This is a simple check: does the single transaction exceed the total budget in SOL?
    // A more sophisticated check might consider the *remaining* budget after previous spends.
    // For now, we check if this spend is <= total budget AND spend <= current balance.
    // This interpretation means the `budget_usdt` is a per-transaction cap, or a total cap if not spent yet.
    // Let's assume `budget_usdt` is the total operational budget the bot instance should not deplete beyond.
    // This means we should check if `current_sol_balance - spending_sol` would leave the account with less than `(initial_total_balance - budget_sol)`.
    // This requires knowing the initial balance or tracking total spent.
    // For simplicity now: the budget_usdt is the *maximum total SOL value* the bot can utilize from the account,
    // relative to its current balance. If current balance is already below budget_sol, it can only spend what it has.

    let effective_spendable_limit_sol = current_sol_balance.min(budget_sol);

    if spending_sol > effective_spendable_limit_sol {
        warn!("Budget Check: Proposed spend ({:.6} SOL) exceeds effective spendable limit derived from budget ({:.6} SOL).", spending_sol, effective_spendable_limit_sol);
        Ok(false)
    } else {
        info!("Budget Check: Proposed spend ({:.6} SOL) is within budget.", spending_sol);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_within_budget() {
        // Price of SOL = $100 USDT
        // Budget = $150 USDT  => 1.5 SOL budget
        // Current Balance = 2.0 SOL
        let sol_price = 100.0;

        // 1. Spend 1.0 SOL: Should be OK.
        assert_eq!(is_within_budget(1.0, 150.0, 2.0, Some(sol_price)).await.unwrap(), true);

        // 2. Spend 1.6 SOL: Should be NOT OK (exceeds 1.5 SOL budget).
        assert_eq!(is_within_budget(1.6, 150.0, 2.0, Some(sol_price)).await.unwrap(), false);

        // 3. Spend 2.1 SOL: Should be NOT OK (exceeds balance, even if budget was higher).
        assert_eq!(is_within_budget(2.1, 300.0, 2.0, Some(sol_price)).await.unwrap(), false);

        // 4. Budget = $50 USDT => 0.5 SOL budget. Current Balance = 0.4 SOL. Spend 0.3 SOL: Should be OK.
        // (Can spend up to min(current_balance, budget_sol) = min(0.4, 0.5) = 0.4 SOL)
        assert_eq!(is_within_budget(0.3, 50.0, 0.4, Some(sol_price)).await.unwrap(), true);

        // 5. Budget = $50 USDT => 0.5 SOL budget. Current Balance = 0.4 SOL. Spend 0.41 SOL: Should be NOT OK.
        assert_eq!(is_within_budget(0.41, 50.0, 0.4, Some(sol_price)).await.unwrap(), false);

        // 6. Budget = $200 USDT => 2.0 SOL budget. Current Balance = 0.5 SOL. Spend 0.5 SOL: Should be OK.
        assert_eq!(is_within_budget(0.5, 200.0, 0.5, Some(sol_price)).await.unwrap(), true);

        // 7. Budget = $200 USDT => 2.0 SOL budget. Current Balance = 0.5 SOL. Spend 0.51 SOL: Should be NOT OK (exceeds balance).
        assert_eq!(is_within_budget(0.51, 200.0, 0.5, Some(sol_price)).await.unwrap(), false);
    }
}
