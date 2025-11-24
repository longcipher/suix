use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::{Result, bail};
use grpc::{GrpcConfig, SuiGrpcClient};
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
    JsonRpc {
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
    /// Make raw gRPC calls (buf curl-like interface)
    Grpc {
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,

        /// gRPC service to call
        #[arg(value_name = "SERVICE")]
        service: String,

        /// gRPC method to call
        #[arg(value_name = "METHOD")]
        method: String,

        /// Parameters for the gRPC call (JSON format)
        #[arg(value_name = "PARAMS")]
        params: Option<String>,

        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,

        /// Output only JSON result for pipeline processing
        #[arg(short = 'j', long)]
        json: bool,

        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// Quick access to common JSON-RPC methods
    #[command(subcommand)]
    JsonRpcQuick(QueryCommands),
    /// Quick access to common gRPC methods (using sui-rpc-api)
    #[command(subcommand)]
    GrpcQuick(GrpcCommands),
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

#[derive(Subcommand)]
enum GrpcCommands {
    /// Get service information
    Info {
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Output only JSON result for pipeline processing
        #[arg(short = 'j', long)]
        json: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// Get object information by ID
    Object {
        /// Object ID to query
        #[arg(value_name = "OBJECT_ID")]
        object_id: String,
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Output only JSON result for pipeline processing
        #[arg(short = 'j', long)]
        json: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// Get transaction by digest
    Tx {
        /// Transaction digest
        #[arg(value_name = "DIGEST")]
        digest: String,
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// Get account balance
    Balance {
        /// Address to query
        #[arg(value_name = "ADDRESS")]
        address: String,
        /// Coin type (optional)
        #[arg(long, value_name = "COIN_TYPE")]
        coin_type: Option<String>,
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// List account balances
    Balances {
        /// Address to query
        #[arg(value_name = "ADDRESS")]
        address: String,
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// Raw gRPC call (similar to buf curl)
    Curl {
        /// gRPC service name
        #[arg(value_name = "SERVICE")]
        service: String,
        /// gRPC method name
        #[arg(value_name = "METHOD")]
        method: String,
        /// Request data as JSON string
        #[arg(short, long, value_name = "JSON")]
        data: Option<String>,
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// List available gRPC methods
    ListMethods {
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
    },
    /// Subscribe to checkpoint stream (supports continuous streaming)
    Subscribe {
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Output only JSON result for pipeline processing
        #[arg(short = 'j', long)]
        json: bool,
        /// Enable continuous streaming mode (polls for new checkpoints)
        #[arg(short = 's', long)]
        stream: bool,
        /// Polling interval in seconds for streaming mode
        #[arg(long, value_name = "SECONDS", default_value = "5")]
        interval: u64,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
    },
    /// Get full checkpoint data
    FullCheckpoint {
        /// Checkpoint sequence number
        #[arg(value_name = "SEQUENCE_NUMBER")]
        sequence_number: u64,
        /// gRPC endpoint URL
        #[arg(
            long,
            value_name = "URL",
            default_value = "https://fullnode.mainnet.sui.io:443"
        )]
        url: String,
        /// Pretty print the response
        #[arg(short, long)]
        pretty: bool,
        /// Request timeout in seconds
        #[arg(long, value_name = "SECONDS", default_value = "30")]
        timeout: u64,
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
        Commands::JsonRpc {
            url,
            method,
            params,
            pretty,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            let config = RpcConfig { url, pretty };
            rt.block_on(make_rpc_call(&config, &method, params.as_deref()))
        }
        Commands::Grpc {
            url,
            service,
            method,
            params: _params,
            pretty,
            json,
            timeout,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(handle_grpc_command(
                url, service, method, pretty, json, timeout,
            ))
        }
        Commands::JsonRpcQuick(query_cmd) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(handle_query_command(query_cmd))
        }
        Commands::GrpcQuick(grpc_cmd) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(handle_grpc2_command(grpc_cmd))
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

async fn handle_grpc_command(
    url: String,
    service: String,
    method: String,
    pretty: bool,
    json: bool,
    timeout: u64,
) -> Result<()> {
    use std::time::Duration;

    use grpc::{GrpcConfig, SuiGrpcClient};

    let config = GrpcConfig {
        url,
        pretty,
        json,
        timeout: Duration::from_secs(timeout),
        headers: vec![],
    };

    let mut client = SuiGrpcClient::new(config)
        .await
        .map_err(|e| eyre::eyre!("Failed to create gRPC client: {}", e))?;

    // Map common service.method combinations
    match (service.as_str(), method.as_str()) {
        ("CheckpointService", "GetLatestCheckpoint") => {
            client
                .get_service_info()
                .await
                .map_err(|e| eyre::eyre!("Failed to get service info: {}", e))?;
        }
        ("ObjectService", "GetObject") => {
            println!("GetObject requires an object ID parameter");
            return Ok(());
        }
        ("CheckpointService", "GetCheckpoint") => {
            println!("GetCheckpoint requires a checkpoint ID parameter");
            return Ok(());
        }
        _ => {
            println!("Service: {service}, Method: {method}");
            println!("This is a placeholder for raw gRPC call functionality");
            println!("You can implement specific method calls here");
        }
    }

    Ok(())
}

async fn handle_grpc2_command(cmd: GrpcCommands) -> Result<()> {
    match cmd {
        GrpcCommands::Info {
            url,
            pretty,
            json,
            timeout,
        } => {
            let config = GrpcConfig {
                url,
                pretty,
                json,
                timeout: std::time::Duration::from_secs(timeout),
                headers: vec![],
            };
            let mut client = SuiGrpcClient::new(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;
            client.get_service_info().await.map_err(|e| eyre::eyre!(e))
        }
        GrpcCommands::Object {
            object_id,
            url,
            pretty,
            json,
            timeout,
        } => {
            let config = GrpcConfig {
                url,
                pretty,
                json,
                timeout: std::time::Duration::from_secs(timeout),
                headers: vec![],
            };
            let mut client = SuiGrpcClient::new(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;
            client
                .get_object(&object_id)
                .await
                .map_err(|e| eyre::eyre!(e))
        }
        GrpcCommands::Tx {
            digest: _digest,
            url: _url,
            pretty: _pretty,
            timeout: _timeout,
        } => {
            println!("gRPC transaction query not yet implemented");
            Ok(())
        }
        GrpcCommands::Balance {
            address: _address,
            coin_type: _coin_type,
            url: _url,
            pretty: _pretty,
            timeout: _timeout,
        } => {
            println!("gRPC balance query not yet implemented");
            Ok(())
        }
        GrpcCommands::Balances {
            address: _address,
            url: _url,
            pretty: _pretty,
            timeout: _timeout,
        } => {
            println!("gRPC balances query not yet implemented");
            Ok(())
        }
        GrpcCommands::Curl {
            service,
            method,
            data,
            url,
            pretty,
            timeout,
        } => {
            let config = GrpcConfig {
                url,
                pretty,
                json: false,
                timeout: std::time::Duration::from_secs(timeout),
                headers: vec![],
            };
            let mut client = SuiGrpcClient::new(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;
            client
                .curl(&service, &method, data.as_deref())
                .await
                .map_err(|e| eyre::eyre!(e))
        }
        GrpcCommands::ListMethods { url } => {
            let config = GrpcConfig {
                url,
                pretty: false,
                json: false,
                timeout: std::time::Duration::from_secs(30),
                headers: vec![],
            };
            let client = SuiGrpcClient::new(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;
            client.show_methods();
            Ok(())
        }
        GrpcCommands::Subscribe {
            url,
            pretty,
            json,
            stream,
            interval,
            timeout,
        } => {
            let config = GrpcConfig {
                url,
                pretty,
                json,
                timeout: std::time::Duration::from_secs(timeout),
                headers: vec![],
            };
            let mut client = SuiGrpcClient::new(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;

            if stream {
                client
                    .subscribe_checkpoints_continuous(interval)
                    .await
                    .map_err(|e| eyre::eyre!(e))
            } else {
                client
                    .subscribe_checkpoints()
                    .await
                    .map_err(|e| eyre::eyre!(e))
            }
        }
        GrpcCommands::FullCheckpoint {
            sequence_number,
            url,
            pretty,
            timeout,
        } => {
            let config = GrpcConfig {
                url,
                pretty,
                json: false,
                timeout: std::time::Duration::from_secs(timeout),
                headers: vec![],
            };
            let mut client = SuiGrpcClient::new(config)
                .await
                .map_err(|e| eyre::eyre!(e))?;
            client
                .get_full_checkpoint(sequence_number)
                .await
                .map_err(|e| eyre::eyre!(e))
        }
    }
}
