pub static PROJECT_NAME: &str = "Skyscope_Solana_MEV_Bot";

pub fn get_env(key: &str) -> String {
    std::env::var(key).unwrap_or(String::from(""))
}

pub fn get_env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_log_level(key: &str, default: log::LevelFilter) -> log::LevelFilter {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<log::LevelFilter>().ok())
        .unwrap_or(default)
}

#[derive(Debug, Clone)]
pub struct Env {
    pub block_engine_url: String,
    pub mainnet_rpc_url: String,
    pub rpc_url_tx: String,
    pub devnet_rpc_url: String,
    pub rpc_url: String,
    pub wss_rpc_url: String,
    pub geyser_url: String,
    pub geyser_access_token: String,
    pub simulator_url: String,
    pub ws_simulator_url: String,
    pub payer_keypair_path: String,
    pub database_name: String,
    pub profit_threshold_sol: f64,
    pub direct_execution: bool,
    pub log_level: log::LevelFilter,
    pub bot_instances: Vec<BotInstanceConfig>,
}

#[derive(Debug, Clone)]
pub struct BotInstanceConfig {
    pub id: usize, // 1-indexed for user-friendliness in config
    pub payer_keypair_path: String,
    pub budget_usdt: f64,
}

impl Env {
    pub fn new() -> Self {
        let mut bot_instances_config = Vec::new();
        let mut multi_instance_mode = false;

        // Check if multi-instance mode is explicitly configured
        if !get_env("BOT_INSTANCE_1_KEYPAIR_PATH").is_empty() {
            multi_instance_mode = true;
        }

        if multi_instance_mode {
            for i in 1..=4 { // Max 4 bot instances
                let keypair_path_env_key = format!("BOT_INSTANCE_{}_KEYPAIR_PATH", i);
                let budget_usdt_env_key = format!("BOT_INSTANCE_{}_BUDGET_USDT", i);

                let payer_keypair_path = get_env(&keypair_path_env_key);

                // If a keypair path is not provided for an instance, stop adding more.
                // This allows users to configure 1, 2, 3, or 4 instances.
                if payer_keypair_path.is_empty() {
                    break;
                }

                let budget_usdt = get_env_f64(&budget_usdt_env_key, 3.0); // Default 3 USDT budget if not specified for this instance

                bot_instances_config.push(BotInstanceConfig {
                    id: i,
                    payer_keypair_path,
                    budget_usdt,
                });
            }
        } else {
            // Single instance mode (legacy or default)
            let main_payer_keypair_path = get_env("PAYER_KEYPAIR_PATH");
            if !main_payer_keypair_path.is_empty() {
                bot_instances_config.push(BotInstanceConfig {
                    id: 0, // Special ID for single/default instance
                    payer_keypair_path: main_payer_keypair_path,
                    budget_usdt: get_env_f64("DEFAULT_BUDGET_USDT", 3.0), // Use a specific env var or default for single instance budget
                });
            }
        }

        Env {
            block_engine_url: get_env("BLOCK_ENGINE_URL"),
            rpc_url: get_env("RPC_URL"),
            mainnet_rpc_url: get_env("MAINNET_RPC_URL"),
            rpc_url_tx: get_env("RPC_URL_TX"),
            devnet_rpc_url: get_env("DEVNET_RPC_URL"),
            wss_rpc_url: get_env("WSS_RPC_URL"),
            geyser_url: get_env("GEYSER_URL"),
            geyser_access_token: get_env("GEYSER_ACCESS_TOKEN"),
            simulator_url: get_env("SIMULATOR_URL"),
            ws_simulator_url: get_env("WS_SIMULATOR_URL"),
            payer_keypair_path: get_env("PAYER_KEYPAIR_PATH"),
            database_name: get_env("DATABASE_NAME"),
            profit_threshold_sol: get_env_f64("PROFIT_THRESHOLD_SOL", 0.02), // Default 0.02 SOL
            direct_execution: get_env_bool("DIRECT_EXECUTION", false), // Default to false (use TCP executor)
            log_level: get_env_log_level("LOG_LEVEL", log::LevelFilter::Info), // Default to Info for project
        }
    }
}

pub static COINBASE: &str = "0xDAFEA492D9c6733ae3d56b7Ed1ADB60692c98Bc5"; // Flashbots Builder

pub static WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub static USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub static USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

/*
Can figure out the balance slot of ERC-20 tokens using the:
EvmSimulator::get_balance_slot method

However, note that this does not work for all tokens.
Especially tokens that are using proxy patterns.
*/
pub static WETH_BALANCE_SLOT: i32 = 3;
pub static USDT_BALANCE_SLOT: i32 = 2;
pub static USDC_BALANCE_SLOT: i32 = 9;

pub static WETH_DECIMALS: u8 = 18;
pub static USDT_DECIMALS: u8 = 6;
pub static USDC_DECIMALS: u8 = 6;
