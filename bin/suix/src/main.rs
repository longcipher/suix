use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::{Result, bail};
use rpc::{RpcConfig, make_rpc_call, methods};
use vanity::{VanityConfig, generate_vanity_addresses};

#[derive(Parser)]
#[command(name = "suix")]
#[command(about = "A comprehensive CLI tool for Sui blockchain operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Sui vanity addresses
    Vanity {
        /// Prefix regex pattern or hex string that the address should start with
        #[arg(long, value_name = "PATTERN")]
        starts_with: Option<String>,

        /// Suffix regex pattern or hex string that the address should end with  
        #[arg(long, value_name = "PATTERN")]
        ends_with: Option<String>,

        /// Path to save the generated vanity contract addresses to (if not specified, prints to terminal)
        #[arg(long, value_name = "PATH")]
        save_path: Option<PathBuf>,

        /// Number of threads to use. Specifying 0 defaults to the number of logical cores
        #[arg(short = 'j', long, value_name = "THREADS", default_value = "0")]
        threads: usize,

        /// Number of vanity addresses to generate before stopping
        #[arg(short = 'n', long, value_name = "COUNT", default_value = "1")]
        count: usize,

        /// Number of addresses to generate per round (affects progress reporting frequency)
        #[arg(long, value_name = "COUNT", default_value = "10000")]
        addresses_per_round: usize,
    },
    /// Make Sui JSON-RPC calls
    Rpc {
        /// RPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,

        /// RPC method to call
        #[arg(value_name = "METHOD")]
        method: String,

        /// Parameters for the RPC call (JSON format)
        #[arg(value_name = "PARAMS")]
        params: Option<String>,

        /// Pretty print the JSON response
        #[arg(short, long)]
        pretty: bool,
    },
    /// Quick access to common RPC methods
    #[command(subcommand)]
    Query(QueryCommands),
}

#[derive(Subcommand)]
enum QueryCommands {
    /// Get chain identifier
    Chain {
        /// RPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the JSON response
        #[arg(short, long)]
        pretty: bool,
    },
    /// Get latest checkpoint sequence number
    Checkpoint {
        /// RPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the JSON response
        #[arg(short, long)]
        pretty: bool,
    },
    /// Get object information by ID
    Object {
        /// Object ID to query
        #[arg(value_name = "OBJECT_ID")]
        object_id: String,
        /// RPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the JSON response
        #[arg(short, long)]
        pretty: bool,
    },
    /// Get transaction by digest
    Tx {
        /// Transaction digest
        #[arg(value_name = "DIGEST")]
        digest: String,
        /// RPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the JSON response
        #[arg(short, long)]
        pretty: bool,
    },
    /// Get account balance
    Balance {
        /// Address to query
        #[arg(value_name = "ADDRESS")]
        address: String,
        /// Coin type (optional)
        #[arg(long, value_name = "COIN_TYPE")]
        coin_type: Option<String>,
        /// RPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the JSON response
        #[arg(short, long)]
        pretty: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Vanity {
            starts_with,
            ends_with,
            save_path,
            threads,
            count,
            addresses_per_round,
        } => {
            // Validate arguments
            if starts_with.is_none() && ends_with.is_none() {
                bail!("At least one of --starts-with or --ends-with must be specified");
            }

            if count == 0 {
                bail!("Count must be greater than 0");
            }

            if addresses_per_round == 0 {
                bail!("Addresses per round must be greater than 0");
            }

            // Ensure save path exists if specified
            if let Some(ref save_path) = save_path {
                std::fs::create_dir_all(save_path)?;
            }

            let config = VanityConfig {
                starts_with,
                ends_with,
                save_path: save_path.map(|p| p.to_string_lossy().to_string()),
                threads,
                max_addresses: count,
                addresses_per_round,
            };

            generate_vanity_addresses(&config)
        }
        Commands::Rpc {
            url,
            method,
            params,
            pretty,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            let config = RpcConfig { url, pretty };
            rt.block_on(make_rpc_call(&config, &method, params.as_deref()))
        }
        Commands::Query(query_cmd) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(handle_query_command(query_cmd))
        }
    }
}

async fn handle_query_command(cmd: QueryCommands) -> Result<()> {
    match cmd {
        QueryCommands::Chain { url, pretty } => {
            let config = RpcConfig { url, pretty };
            methods::get_chain_identifier(&config).await
        }
        QueryCommands::Checkpoint { url, pretty } => {
            let config = RpcConfig { url, pretty };
            methods::get_latest_checkpoint_sequence_number(&config).await
        }
        QueryCommands::Object {
            object_id,
            url,
            pretty,
        } => {
            let config = RpcConfig { url, pretty };
            methods::get_object(&config, &object_id).await
        }
        QueryCommands::Tx {
            digest,
            url,
            pretty,
        } => {
            let config = RpcConfig { url, pretty };
            methods::get_transaction_block(&config, &digest).await
        }
        QueryCommands::Balance {
            address,
            coin_type,
            url,
            pretty,
        } => {
            let config = RpcConfig { url, pretty };
            methods::get_balance(&config, &address, coin_type.as_deref()).await
        }
    }
}
