use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::Result;

pub const MAINNET_RPC: &str = "https://fullnode.mainnet.sui.io:443";
pub const TESTNET_RPC: &str = "https://fullnode.testnet.sui.io:443";
pub const DEVNET_RPC: &str = "https://fullnode.devnet.sui.io:443";
pub const LOCALNET_RPC: &str = "http://127.0.0.1:9000";

pub const MAINNET_GRPC: &str = "https://fullnode.mainnet.sui.io:443";
pub const TESTNET_GRPC: &str = "https://fullnode.testnet.sui.io:443";
pub const DEVNET_GRPC: &str = "https://fullnode.devnet.sui.io:443";
pub const LOCALNET_GRPC: &str = "http://127.0.0.1:9000";

pub const TESTNET_FAUCET: &str = "https://faucet.testnet.sui.io/gas";
pub const DEVNET_FAUCET: &str = "https://faucet.devnet.sui.io/gas";
pub const LOCALNET_FAUCET: &str = "http://127.0.0.1:9123/gas";

#[derive(Parser)]
#[command(name = "suix")]
#[command(about = "A comprehensive CLI tool for Sui blockchain operations")]
#[command(version)]
pub struct Cli {
    /// Network profile: mainnet, testnet, devnet, localnet. Overrides --url defaults.
    #[arg(long, global = true, value_name = "PROFILE")]
    pub profile: Option<String>,

    /// Config file for profiles and defaults.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

fn url_default(default: &str) -> String {
    std::env::var("SUIX_URL").unwrap_or_else(|_| default.to_string())
}

pub fn rpc_url_default() -> String {
    url_default(MAINNET_RPC)
}

pub fn grpc_url_default() -> String {
    std::env::var("SUIX_GRPC_URL").unwrap_or_else(|_| MAINNET_GRPC.to_string())
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate Sui vanity addresses
    Vanity(VanityArgs),
    /// Make Sui JSON-RPC calls
    JsonRpc(JsonRpcArgs),
    /// Make raw gRPC calls
    Grpc(GrpcArgs),
    /// Quick access to common JSON-RPC methods
    #[command(subcommand)]
    JsonRpcQuick(QueryCommands),
    /// Quick access to common gRPC methods (using sui-rpc-api)
    #[command(subcommand)]
    GrpcQuick(GrpcCommands),
    /// Local key management
    #[command(subcommand)]
    Key(KeyCommands),
    /// Read-only chain queries
    #[command(subcommand)]
    Query(ReadCommands),
    /// Coin queries
    #[command(subcommand)]
    Coin(CoinCommands),
    /// Unsigned transaction builders (unsafe_* API)
    #[command(subcommand)]
    Tx(TxCommands),
    /// Staking operations
    #[command(subcommand)]
    Stake(StakeCommands),
    /// System and governance queries
    #[command(subcommand)]
    System(SystemCommands),
    /// Small utilities
    #[command(subcommand)]
    Util(UtilCommands),
    /// GraphQL passthrough
    Graphql(GraphqlArgs),
    /// Generate shell completions
    Completion(CompletionArgs),
}

#[derive(Parser)]
pub struct VanityArgs {
    /// Prefix pattern that the address should start with
    #[arg(long, value_name = "PATTERN")]
    pub starts_with: Option<String>,
    /// Suffix pattern that the address should end with
    #[arg(long, value_name = "PATTERN")]
    pub ends_with: Option<String>,
    /// Infix pattern that the address should contain
    #[arg(long, value_name = "PATTERN")]
    pub contains: Option<String>,
    /// Signature scheme: ed25519, secp256k1, secp256r1
    #[arg(long, value_name = "SCHEME", default_value = "ed25519")]
    pub scheme: String,
    /// Only estimate difficulty without generating
    #[arg(long)]
    pub estimate_only: bool,
    /// Path to save generated addresses to (otherwise prints to terminal)
    #[arg(long, value_name = "PATH")]
    pub save_path: Option<PathBuf>,
    /// Number of threads (0 = number of logical cores)
    #[arg(short = 'j', long, value_name = "THREADS", default_value = "0")]
    pub threads: usize,
    /// Number of vanity addresses to generate before stopping
    #[arg(short = 'n', long, value_name = "COUNT", default_value = "1")]
    pub count: usize,
    /// Number of addresses to generate per round
    #[arg(long, value_name = "COUNT", default_value = "10000")]
    pub addresses_per_round: usize,
}

#[derive(Parser)]
pub struct JsonRpcArgs {
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// RPC method to call
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// Parameters for the RPC call (JSON format)
    #[arg(value_name = "PARAMS")]
    pub params: Option<String>,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct GrpcArgs {
    /// gRPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = grpc_url_default())]
    pub url: String,
    /// gRPC service to call
    #[arg(value_name = "SERVICE")]
    pub service: String,
    /// gRPC method to call
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// Parameters for the gRPC call (JSON format)
    #[arg(value_name = "PARAMS")]
    pub params: Option<String>,
    /// Pretty print the response
    #[arg(short, long)]
    pub pretty: bool,
    /// Output only JSON result for pipeline processing
    #[arg(short = 'j', long)]
    pub json: bool,
    /// Request timeout in seconds
    #[arg(long, value_name = "SECONDS", default_value = "30")]
    pub timeout: u64,
    /// Extra request header (repeatable, KEY=VALUE)
    #[arg(long, value_name = "KEY=VALUE")]
    pub header: Vec<String>,
}

#[derive(Subcommand)]
pub enum QueryCommands {
    /// Get chain identifier
    Chain(CommonQueryArgs),
    /// Get latest checkpoint sequence number
    Checkpoint(CommonQueryArgs),
    /// Get object information by ID
    Object(ObjectArgs),
    /// Get transaction by digest
    Tx(TxQueryArgs),
    /// Get account balance
    Balance(BalanceArgs),
    /// Get all balances of an address
    Balances(AddressQueryArgs),
    /// Get coins owned by an address
    Coins(CoinsArgs),
    /// Get all coins owned by an address
    AllCoins(AllCoinsArgs),
    /// Get objects owned by an address
    Owned(OwnedArgs),
    /// Get dynamic fields of a parent object
    DynamicFields(DynamicFieldsArgs),
    /// Query events
    Events(EventsArgs),
    /// Get staked SUI for an owner
    Stakes(AddressQueryArgs),
    /// Try to get a past object version
    PastObject(PastObjectArgs),
    /// Get a checkpoint by sequence number or digest
    CheckpointById(CheckpointByIdArgs),
    /// Get a page of checkpoints
    Checkpoints(PageArgs),
    /// Query transaction blocks
    Txs(QueryTxsArgs),
    /// Get coin metadata
    CoinMetadata(CoinTypeArgs),
    /// Get total coin supply
    TotalSupply(CoinTypeArgs),
    /// Get normalized Move modules of a package
    Package(PackageArgs),
    /// Get Move function argument types
    FunctionArgs(FunctionArgsArgs),
    /// Get protocol config
    Protocol(ProtocolArgs),
    /// Dry-run unsigned transaction bytes
    DryRun(DryRunArgs),
    /// Dev-inspect a move call
    DevInspect(DevInspectArgs),
}

#[derive(Parser)]
pub struct CommonQueryArgs {
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct ObjectArgs {
    /// Object ID to query
    #[arg(value_name = "OBJECT_ID")]
    pub object_id: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct TxQueryArgs {
    /// Transaction digest
    #[arg(value_name = "DIGEST")]
    pub digest: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct BalanceArgs {
    /// Address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Coin type (optional)
    #[arg(long, value_name = "COIN_TYPE")]
    pub coin_type: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct AddressQueryArgs {
    /// Address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct CoinsArgs {
    /// Address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Coin type filter
    #[arg(long, value_name = "COIN_TYPE")]
    pub coin_type: Option<String>,
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct AllCoinsArgs {
    /// Address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct OwnedArgs {
    /// Owner address
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct DynamicFieldsArgs {
    /// Parent object ID
    #[arg(value_name = "PARENT_ID")]
    pub parent_id: String,
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct EventsArgs {
    /// Event filter as JSON (default matches all recent events)
    #[arg(long, value_name = "JSON")]
    pub filter: Option<String>,
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct PastObjectArgs {
    /// Object ID
    #[arg(value_name = "OBJECT_ID")]
    pub object_id: String,
    /// Object version
    #[arg(value_name = "VERSION")]
    pub version: u64,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct CheckpointByIdArgs {
    /// Checkpoint sequence number or digest
    #[arg(value_name = "ID")]
    pub id: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct PageArgs {
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct QueryTxsArgs {
    /// Transaction filter as JSON
    #[arg(long, value_name = "JSON")]
    pub filter: Option<String>,
    /// Page limit
    #[arg(long, default_value = "20")]
    pub limit: u32,
    /// Page cursor
    #[arg(long)]
    pub cursor: Option<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct CoinTypeArgs {
    /// Coin type, e.g. 0x2::sui::SUI
    #[arg(value_name = "COIN_TYPE")]
    pub coin_type: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct PackageArgs {
    /// Package object ID
    #[arg(value_name = "PACKAGE_ID")]
    pub package_id: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct FunctionArgsArgs {
    /// Package object ID
    #[arg(value_name = "PACKAGE_ID")]
    pub package_id: String,
    /// Module name
    #[arg(value_name = "MODULE")]
    pub module: String,
    /// Function name
    #[arg(value_name = "FUNCTION")]
    pub function: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct ProtocolArgs {
    /// Protocol version (default: latest)
    #[arg(long)]
    pub version: Option<u64>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct DryRunArgs {
    /// Base64 unsigned transaction bytes
    #[arg(long)]
    pub tx_bytes: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct DevInspectArgs {
    /// Sender address
    #[arg(long)]
    pub sender: String,
    /// Package object ID
    #[arg(long)]
    pub package: String,
    /// Module name
    #[arg(long)]
    pub module: String,
    /// Function name
    #[arg(long)]
    pub function: String,
    /// Type arguments as JSON array
    #[arg(long, default_value = "[]")]
    pub type_args: String,
    /// Call arguments as JSON array
    #[arg(long, default_value = "[]")]
    pub args: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct GrpcEndpointArgs {
    /// gRPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = grpc_url_default())]
    pub url: String,
    /// Pretty print the response
    #[arg(short, long)]
    pub pretty: bool,
    /// Output only JSON result for pipeline processing
    #[arg(short = 'j', long)]
    pub json: bool,
    /// Request timeout in seconds
    #[arg(long, value_name = "SECONDS", default_value = "30")]
    pub timeout: u64,
    /// Extra request header (repeatable, KEY=VALUE)
    #[arg(long, value_name = "KEY=VALUE")]
    pub header: Vec<String>,
}

#[derive(Parser)]
pub struct GrpcObjectArgs {
    /// Object ID to query
    #[arg(value_name = "OBJECT_ID")]
    pub object_id: String,
    /// Object version (optional)
    #[arg(long)]
    pub version: Option<u64>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcTxArgs {
    /// Transaction digest
    #[arg(value_name = "DIGEST")]
    pub digest: String,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcAddressArgs {
    /// Address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcBalanceArgs {
    /// Address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Coin type (default: 0x2::sui::SUI)
    #[arg(long, value_name = "COIN_TYPE")]
    pub coin_type: Option<String>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcOwnedArgs {
    /// Owner address
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Object type filter
    #[arg(long, value_name = "OBJECT_TYPE")]
    pub object_type: Option<String>,
    /// Page size
    #[arg(long)]
    pub page_size: Option<u32>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcParentArgs {
    /// Parent object ID
    #[arg(value_name = "PARENT_ID")]
    pub parent_id: String,
    /// Page size
    #[arg(long)]
    pub page_size: Option<u32>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcCurlArgs {
    /// gRPC service name
    #[arg(value_name = "SERVICE")]
    pub service: String,
    /// gRPC method name
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// Request data as JSON string
    #[arg(short, long, value_name = "JSON")]
    pub data: Option<String>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcSubscribeArgs {
    /// Enable continuous streaming mode
    #[arg(short = 's', long)]
    pub stream: bool,
    /// Polling interval in seconds for fallback polling mode
    #[arg(long, value_name = "SECONDS", default_value = "5")]
    pub interval: u64,
    /// Max checkpoints to print in real-stream mode (default: 5 for one-shot)
    #[arg(long)]
    pub limit: Option<u64>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcCheckpointArgs {
    /// Checkpoint sequence number
    #[arg(value_name = "SEQUENCE_NUMBER")]
    pub sequence_number: u64,
    /// Dump raw checkpoint to a directory for offline analysis
    #[arg(long, value_name = "DIR")]
    pub dump_dir: Option<PathBuf>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcSimulateArgs {
    /// Base64 unsigned transaction bytes
    #[arg(long)]
    pub tx_bytes: String,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Parser)]
pub struct GrpcExecuteArgs {
    /// Base64 unsigned transaction bytes
    #[arg(long)]
    pub tx_bytes: String,
    /// Base64 signatures (repeatable)
    #[arg(long = "signature", value_name = "SIG")]
    pub signatures: Vec<String>,
    /// Wait for checkpoint inclusion
    #[arg(long)]
    pub wait: bool,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Subcommand)]
pub enum GrpcCommands {
    /// Get service information (latest checkpoint)
    Info(GrpcEndpointArgs),
    /// Get object information by ID
    Object(GrpcObjectArgs),
    /// Get transaction by digest
    Tx(GrpcTxArgs),
    /// Get account balance
    Balance(GrpcBalanceArgs),
    /// List account balances
    Balances(GrpcAddressArgs),
    /// List objects owned by an address
    Owned(GrpcOwnedArgs),
    /// Get dynamic fields of a parent object
    DynamicFields(GrpcParentArgs),
    /// Get chain identifier
    Chain(GrpcEndpointArgs),
    /// Get reference gas price
    GasPrice(GrpcEndpointArgs),
    /// Get delegated stakes for an address
    Stakes(GrpcAddressArgs),
    /// Get system state summary
    SystemState(SystemStateArgs),
    /// Simulate unsigned transaction bytes
    Simulate(GrpcSimulateArgs),
    /// Execute a signed transaction
    Execute(GrpcExecuteArgs),
    /// Raw gRPC call (similar to buf curl)
    Curl(GrpcCurlArgs),
    /// List available gRPC methods
    ListMethods(GrpcListMethodsArgs),
    /// Subscribe to checkpoint stream
    Subscribe(GrpcSubscribeArgs),
    /// Get full checkpoint data
    FullCheckpoint(GrpcCheckpointArgs),
}

#[derive(Parser)]
pub struct GrpcListMethodsArgs {
    /// gRPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = grpc_url_default())]
    pub url: String,
}

#[derive(Parser)]
pub struct SystemStateArgs {
    /// Epoch (default: latest)
    #[arg(long)]
    pub epoch: Option<u64>,
    #[clap(flatten)]
    pub endpoint: GrpcEndpointArgs,
}

#[derive(Subcommand)]
pub enum KeyCommands {
    /// Generate a new keypair
    Generate(KeyGenerateArgs),
    /// Import a key into the suix keystore
    Import(KeyImportArgs),
    /// Export a key from the suix keystore
    Export(KeyExportArgs),
    /// List keys in the suix keystore
    List(KeyListArgs),
    /// Sign raw bytes (hex) with a key
    Sign(KeySignArgs),
    /// Sign unsigned transaction bytes with a key
    SignTx(KeySignTxArgs),
    /// Decode a base64 signature
    DecodeSig(KeyDecodeSigArgs),
    /// Build a multisig address
    MultisigAddress(KeyMultisigArgs),
}

#[derive(Parser)]
pub struct KeyGenerateArgs {
    /// Signature scheme: ed25519, secp256k1, secp256r1
    #[arg(long, default_value = "ed25519")]
    pub scheme: String,
    /// Mnemonic word length: word12/15/18/21/24
    #[arg(long, value_name = "WORD_LENGTH")]
    pub word_length: Option<String>,
}

#[derive(Parser)]
pub struct KeyImportArgs {
    /// Private key (suiprivkey bech32 or base64). Reads SUIX_KEY if absent.
    #[arg(long)]
    pub secret: Option<String>,
    /// Alias for the key
    #[arg(long)]
    pub alias: Option<String>,
    /// Keystore file path (default: platform data dir)
    #[arg(long, value_name = "PATH")]
    pub keystore: Option<PathBuf>,
}

#[derive(Parser)]
pub struct KeyExportArgs {
    /// Address to export
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Keystore file path (default: platform data dir)
    #[arg(long, value_name = "PATH")]
    pub keystore: Option<PathBuf>,
}

#[derive(Parser)]
pub struct KeyListArgs {
    /// Keystore file path (default: platform data dir)
    #[arg(long, value_name = "PATH")]
    pub keystore: Option<PathBuf>,
}

#[derive(Parser)]
pub struct KeySourceArgs {
    /// Private key (suiprivkey bech32 or base64). Reads SUIX_KEY if absent.
    #[arg(long)]
    pub secret: Option<String>,
    /// Read private key from a file
    #[arg(long, value_name = "PATH", conflicts_with = "secret")]
    pub secret_file: Option<PathBuf>,
    /// Keystore file path, combined with --address to load the key
    #[arg(long, value_name = "PATH")]
    pub keystore: Option<PathBuf>,
    /// Address in the keystore to use (requires --keystore)
    #[arg(long, value_name = "ADDRESS")]
    pub address: Option<String>,
}

#[derive(Parser)]
pub struct KeySignArgs {
    /// Message as hex string
    #[arg(long)]
    pub message: String,
    #[clap(flatten)]
    pub key: KeySourceArgs,
}

#[derive(Parser)]
pub struct KeySignTxArgs {
    /// Base64 unsigned transaction bytes
    #[arg(long)]
    pub tx_bytes: String,
    #[clap(flatten)]
    pub key: KeySourceArgs,
}

#[derive(Parser)]
pub struct KeyDecodeSigArgs {
    /// Base64 signature
    #[arg(value_name = "SIGNATURE")]
    pub signature: String,
}

#[derive(Parser)]
pub struct KeyMultisigArgs {
    /// Base64 public keys (repeatable)
    #[arg(long = "pk", value_name = "PUBKEY")]
    pub pks: Vec<String>,
    /// Weights (repeatable, same order as --pk)
    #[arg(long = "weight", value_name = "WEIGHT")]
    pub weights: Vec<u8>,
    /// Signature threshold
    #[arg(long)]
    pub threshold: u16,
}

#[derive(Subcommand)]
pub enum ReadCommands {
    /// Get object information by ID
    Object(ObjectArgs),
    /// Get objects owned by an address
    Owned(OwnedArgs),
    /// Get a past object version
    PastObject(PastObjectArgs),
    /// Get dynamic fields of a parent object
    DynamicFields(DynamicFieldsArgs),
    /// Get transaction by digest
    Tx(TxQueryArgs),
    /// Query transaction blocks
    Txs(QueryTxsArgs),
    /// Query events
    Events(EventsArgs),
    /// Get a checkpoint by sequence number or digest
    Checkpoint(CheckpointByIdArgs),
    /// Get a page of checkpoints
    Checkpoints(PageArgs),
    /// Get normalized Move modules of a package
    Package(PackageArgs),
    /// Get Move function argument types
    FunctionArgs(FunctionArgsArgs),
}

#[derive(Subcommand)]
pub enum CoinCommands {
    /// Get account balance for a coin type
    Balance(BalanceArgs),
    /// Get all balances of an address
    Balances(AddressQueryArgs),
    /// Get coins owned by an address
    Coins(CoinsArgs),
    /// Get all coins owned by an address
    AllCoins(AllCoinsArgs),
    /// Get coin metadata
    Metadata(CoinTypeArgs),
    /// Get total coin supply
    Supply(CoinTypeArgs),
}

#[derive(Parser)]
pub struct TxBaseArgs {
    /// Signer address
    #[arg(long)]
    pub signer: String,
    /// Gas object ID (default: auto-select)
    #[arg(long, value_name = "OBJECT_ID")]
    pub gas: Option<String>,
    /// Gas budget in MIST
    #[arg(long, default_value = "50000000")]
    pub gas_budget: u64,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Also dry-run the built transaction
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum TxCommands {
    /// Build unsigned transfer-object transaction
    Transfer(TransferArgs),
    /// Build unsigned transfer-sui transaction
    TransferSui(TransferSuiArgs),
    /// Build unsigned pay transaction
    Pay(PayArgs),
    /// Build unsigned pay-sui transaction
    PaySui(PaySuiArgs),
    /// Build unsigned pay-all-sui transaction
    PayAllSui(PayAllSuiArgs),
    /// Build unsigned move-call transaction
    Call(CallArgs),
    /// Build unsigned split-coin transaction
    Split(SplitArgs),
    /// Build unsigned split-coin-equal transaction
    SplitEqual(SplitEqualArgs),
    /// Build unsigned merge-coins transaction
    Merge(MergeArgs),
    /// Build unsigned publish transaction
    Publish(PublishArgs),
    /// Submit a signed transaction
    Submit(SubmitArgs),
    /// Dry-run unsigned transaction bytes
    DryRun(DryRunArgs),
    /// Wait for a transaction to confirm
    Wait(WaitArgs),
}

#[derive(Parser)]
pub struct TransferArgs {
    /// Object ID to transfer
    #[arg(long)]
    pub object_id: String,
    /// Recipient address
    #[arg(long)]
    pub to: String,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct TransferSuiArgs {
    /// SUI coin object ID
    #[arg(long)]
    pub coin: String,
    /// Recipient address
    #[arg(long)]
    pub to: String,
    /// Amount in MIST (default: whole coin)
    #[arg(long)]
    pub amount: Option<u64>,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct PayArgs {
    /// Input coin object IDs (repeatable)
    #[arg(long = "coin", value_name = "OBJECT_ID")]
    pub coins: Vec<String>,
    /// Recipient addresses (repeatable, same order as --amount)
    #[arg(long = "to", value_name = "ADDRESS")]
    pub recipients: Vec<String>,
    /// Amounts in MIST (repeatable, same order as --to)
    #[arg(long = "amount", value_name = "MIST")]
    pub amounts: Vec<u64>,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct PaySuiArgs {
    /// Input SUI coin object IDs (repeatable)
    #[arg(long = "coin", value_name = "OBJECT_ID")]
    pub coins: Vec<String>,
    /// Recipient addresses (repeatable, same order as --amount)
    #[arg(long = "to", value_name = "ADDRESS")]
    pub recipients: Vec<String>,
    /// Amounts in MIST (repeatable, same order as --to)
    #[arg(long = "amount", value_name = "MIST")]
    pub amounts: Vec<u64>,
    /// Gas budget in MIST
    #[arg(long, default_value = "50000000")]
    pub gas_budget: u64,
    /// Signer address
    #[arg(long)]
    pub signer: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Also dry-run the built transaction
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser)]
pub struct PayAllSuiArgs {
    /// Input SUI coin object IDs (repeatable)
    #[arg(long = "coin", value_name = "OBJECT_ID")]
    pub coins: Vec<String>,
    /// Recipient address
    #[arg(long)]
    pub to: String,
    /// Gas budget in MIST
    #[arg(long, default_value = "50000000")]
    pub gas_budget: u64,
    /// Signer address
    #[arg(long)]
    pub signer: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Also dry-run the built transaction
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser)]
pub struct CallArgs {
    /// Package object ID
    #[arg(long)]
    pub package: String,
    /// Module name
    #[arg(long)]
    pub module: String,
    /// Function name
    #[arg(long)]
    pub function: String,
    /// Type arguments (repeatable)
    #[arg(long = "type-arg", value_name = "TYPE")]
    pub type_args: Vec<String>,
    /// Call arguments as JSON array
    #[arg(long, default_value = "[]")]
    pub args: String,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct SplitArgs {
    /// Coin object ID to split
    #[arg(long)]
    pub coin: String,
    /// Amounts in MIST (repeatable)
    #[arg(long = "amount", value_name = "MIST")]
    pub amounts: Vec<u64>,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct SplitEqualArgs {
    /// Coin object ID to split
    #[arg(long)]
    pub coin: String,
    /// Number of equal coins
    #[arg(long)]
    pub count: u64,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct MergeArgs {
    /// Primary coin object ID (survives)
    #[arg(long)]
    pub primary: String,
    /// Coin object ID to merge (destroyed)
    #[arg(long)]
    pub merge: String,
    #[clap(flatten)]
    pub base: TxBaseArgs,
}

#[derive(Parser)]
pub struct PublishArgs {
    /// Sender address
    #[arg(long)]
    pub sender: String,
    /// Compiled module bytes as base64 (repeatable)
    #[arg(long = "module", value_name = "BASE64")]
    pub modules: Vec<String>,
    /// Transitive dependency object IDs (repeatable)
    #[arg(long = "dep", value_name = "OBJECT_ID")]
    pub dependencies: Vec<String>,
    /// Gas object ID (default: auto-select)
    #[arg(long, value_name = "OBJECT_ID")]
    pub gas: Option<String>,
    /// Gas budget in MIST
    #[arg(long, default_value = "50000000")]
    pub gas_budget: u64,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Also dry-run the built transaction
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser)]
pub struct SubmitArgs {
    /// Base64 unsigned transaction bytes
    #[arg(long)]
    pub tx_bytes: String,
    /// Base64 signatures (repeatable)
    #[arg(long = "signature", value_name = "SIG")]
    pub signatures: Vec<String>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct WaitArgs {
    /// Transaction digest
    #[arg(value_name = "DIGEST")]
    pub digest: String,
    /// Timeout in seconds
    #[arg(long, default_value = "60")]
    pub timeout: u64,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Subcommand)]
pub enum StakeCommands {
    /// Build unsigned delegate transaction
    Delegate(DelegateArgs),
    /// Build unsigned undelegate transaction
    Undelegate(UndelegateArgs),
    /// Show delegated stakes for an address
    Rewards(AddressQueryArgs),
}

#[derive(Parser)]
pub struct DelegateArgs {
    /// Signer address
    #[arg(long)]
    pub signer: String,
    /// SUI coin object IDs to stake (repeatable)
    #[arg(long = "coin", value_name = "OBJECT_ID")]
    pub coins: Vec<String>,
    /// Amount in MIST (default: whole coins)
    #[arg(long)]
    pub amount: Option<u64>,
    /// Validator address
    #[arg(long)]
    pub validator: String,
    /// Gas object ID (default: auto-select)
    #[arg(long, value_name = "OBJECT_ID")]
    pub gas: Option<String>,
    /// Gas budget in MIST
    #[arg(long, default_value = "50000000")]
    pub gas_budget: u64,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Also dry-run the built transaction
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser)]
pub struct UndelegateArgs {
    /// Signer address
    #[arg(long)]
    pub signer: String,
    /// StakedSui object ID
    #[arg(long)]
    pub staked_sui: String,
    /// Gas object ID (default: auto-select)
    #[arg(long, value_name = "OBJECT_ID")]
    pub gas: Option<String>,
    /// Gas budget in MIST
    #[arg(long, default_value = "50000000")]
    pub gas_budget: u64,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Also dry-run the built transaction
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum SystemCommands {
    /// Get chain identifier
    Chain(CommonQueryArgs),
    /// Get reference gas price
    GasPrice(CommonQueryArgs),
    /// Get committee info
    Committee(CommitteeArgs),
    /// Get latest SUI system state
    State(CommonQueryArgs),
    /// Get validators APY table
    Apy(CommonQueryArgs),
    /// Get current epoch
    Epoch(CommonQueryArgs),
    /// Get protocol config
    Protocol(ProtocolArgs),
    /// Get total transaction count
    TotalTxs(CommonQueryArgs),
}

#[derive(Parser)]
pub struct CommitteeArgs {
    /// Epoch (default: latest)
    #[arg(long)]
    pub epoch: Option<u64>,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Subcommand)]
pub enum UtilCommands {
    /// Decode base64 transaction bytes into TransactionData debug
    DecodeTx(DecodeTxArgs),
    /// Convert addresses between formats
    Address(AddrArgs),
    /// Check RPC and gRPC health of an endpoint
    Health(HealthArgs),
    /// Request gas from a faucet
    Faucet(FaucetArgs),
    /// Resolve a SuiNS name to an address
    Resolve(ResolveArgs),
    /// Reverse-resolve an address to SuiNS names
    Reverse(ReverseArgs),
}

#[derive(Parser)]
pub struct DecodeTxArgs {
    /// Base64 unsigned transaction bytes
    #[arg(long)]
    pub tx_bytes: String,
}

#[derive(Parser)]
pub struct AddrArgs {
    /// Address to format
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// Print the full address instead of shortened form
    #[arg(long)]
    pub long: bool,
}

#[derive(Parser)]
pub struct HealthArgs {
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// gRPC endpoint URL (default: same as --url)
    #[arg(long, value_name = "URL")]
    pub grpc_url: Option<String>,
    /// Request timeout in seconds
    #[arg(long, default_value = "10")]
    pub timeout: u64,
}

#[derive(Parser)]
pub struct FaucetArgs {
    /// Address to fund
    #[arg(long)]
    pub address: String,
    /// Faucet URL (default by --profile)
    #[arg(long, value_name = "URL")]
    pub url: Option<String>,
}

#[derive(Parser)]
pub struct ResolveArgs {
    /// SuiNS name, e.g. example.sui
    #[arg(value_name = "NAME")]
    pub name: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct ReverseArgs {
    /// Address to reverse-resolve
    #[arg(value_name = "ADDRESS")]
    pub address: String,
    /// RPC endpoint URL
    #[arg(long, value_name = "URL", default_value_t = rpc_url_default())]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Parser)]
pub struct GraphqlArgs {
    /// GraphQL query string
    #[arg(long)]
    pub query: String,
    /// Variables as JSON object string
    #[arg(long, default_value = "{}")]
    pub variables: String,
    /// GraphQL endpoint URL
    #[arg(
        long,
        value_name = "URL",
        default_value = "https://graphql.mainnet.sui.io/graphql"
    )]
    pub url: String,
    /// Pretty print the JSON response
    #[arg(short, long)]
    pub pretty: bool,
    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,
}

#[derive(Parser)]
pub struct CompletionArgs {
    /// Shell: bash, zsh, fish, powershell, elvish
    #[arg(value_name = "SHELL")]
    pub shell: String,
}

pub fn apply_profile(profile: Option<&str>, rpc_url: &mut String, grpc_url: Option<&mut String>) {
    let Some(profile) = profile else { return };
    match profile {
        "mainnet" => {
            *rpc_url = MAINNET_RPC.to_string();
            if let Some(g) = grpc_url {
                *g = MAINNET_GRPC.to_string();
            }
        }
        "testnet" => {
            *rpc_url = TESTNET_RPC.to_string();
            if let Some(g) = grpc_url {
                *g = TESTNET_GRPC.to_string();
            }
        }
        "devnet" => {
            *rpc_url = DEVNET_RPC.to_string();
            if let Some(g) = grpc_url {
                *g = DEVNET_GRPC.to_string();
            }
        }
        "localnet" | "local" => {
            *rpc_url = LOCALNET_RPC.to_string();
            if let Some(g) = grpc_url {
                *g = LOCALNET_GRPC.to_string();
            }
        }
        _ => {}
    }
}

pub fn faucet_url_for_profile(profile: Option<&str>) -> &'static str {
    match profile {
        Some("devnet") => DEVNET_FAUCET,
        Some("localnet") | Some("local") => LOCALNET_FAUCET,
        _ => TESTNET_FAUCET,
    }
}

pub fn resolve_key_source(key: &KeySourceArgs) -> Result<keystore::SuiKeyPairReexport> {
    if let Some(secret) = &key.secret {
        return keystore::load_keypair(secret);
    }
    if let Some(path) = &key.secret_file {
        return keystore::load_keypair_from_file(path);
    }
    if let (Some(store), Some(address)) = (&key.keystore, &key.address) {
        use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
        let store = FileBasedKeystore::load_or_create(store)
            .map_err(|e| eyre::eyre!("Failed to open keystore: {}", e))?;
        let address: sui_types::base_types::SuiAddress = address
            .parse()
            .map_err(|e| eyre::eyre!("Invalid address: {}", e))?;
        return store
            .export(&address)
            .map(|kp| kp.copy())
            .map_err(|e| eyre::eyre!("Failed to export key: {}", e));
    }
    if let Ok(secret) = std::env::var("SUIX_KEY") {
        return keystore::load_keypair(&secret);
    }
    Err(eyre::eyre!(
        "No key provided; use --secret, --secret-file, --keystore+--address, or SUIX_KEY"
    ))
}

pub fn parse_headers(headers: &[String]) -> Result<Vec<(String, String)>> {
    let mut out = Vec::with_capacity(headers.len());
    for h in headers {
        let (k, v) = h
            .split_once('=')
            .ok_or_else(|| eyre::eyre!("Invalid header '{h}'; expected KEY=VALUE"))?;
        out.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(out)
}

pub fn grpc_config_from(
    endpoint: &GrpcEndpointArgs,
    profile: Option<&str>,
) -> Result<grpc::GrpcConfig> {
    use std::time::Duration;

    let mut url = endpoint.url.clone();
    if profile.is_some() {
        let mut rpc = String::new();
        let mut grpc_url = endpoint.url.clone();
        apply_profile(profile, &mut rpc, Some(&mut grpc_url));
        if endpoint.url == grpc_url_default() {
            url = grpc_url;
        }
    }
    Ok(grpc::GrpcConfig {
        url,
        pretty: endpoint.pretty,
        json: endpoint.json,
        timeout: Duration::from_secs(endpoint.timeout),
        headers: parse_headers(&endpoint.header)?,
    })
}

pub fn rpc_config_from_url(url: &str, pretty: bool, profile: Option<&str>) -> rpc::RpcConfig {
    let mut resolved = url.to_string();
    if profile.is_some() {
        let mut grpc_dummy = String::new();
        apply_profile(profile, &mut resolved, Some(&mut grpc_dummy));
        if url == rpc_url_default() {
            // apply_profile already set the profile URL
        }
    }
    rpc::RpcConfig {
        url: resolved,
        pretty,
    }
}

pub fn check_pretty_json(pretty: bool, json: bool) -> Result<()> {
    if pretty && json {
        eyre::bail!("--pretty and --json are mutually exclusive");
    }
    Ok(())
}

pub fn shorten_address(address: &str) -> String {
    let stripped = address.strip_prefix("0x").unwrap_or(address);
    if stripped.len() <= 12 {
        return address.to_string();
    }
    format!("0x{}...{}", &stripped[..6], &stripped[stripped.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_address() {
        let full = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert_eq!(shorten_address(full), "0x123456...cdef");
        assert_eq!(shorten_address("0x1234"), "0x1234");
    }

    #[test]
    fn test_parse_headers() {
        let headers = parse_headers(&["Authorization=Bearer abc".to_string()]).unwrap();
        assert_eq!(
            headers,
            vec![("Authorization".to_string(), "Bearer abc".to_string())]
        );
        assert!(parse_headers(&["no-equals".to_string()]).is_err());
    }

    #[test]
    fn test_faucet_url_for_profile() {
        assert_eq!(faucet_url_for_profile(Some("devnet")), DEVNET_FAUCET);
        assert_eq!(faucet_url_for_profile(Some("localnet")), LOCALNET_FAUCET);
        assert_eq!(faucet_url_for_profile(Some("mainnet")), TESTNET_FAUCET);
        assert_eq!(faucet_url_for_profile(None), TESTNET_FAUCET);
    }
}
