# Suix - Comprehensive Sui CLI Tool

A high-performance, multi-purpose CLI tool for Sui blockchain operations, featuring vanity address generation, JSON-RPC calls, and native gRPC support with real-time streaming capabilities.

## ✨ Features

- 🏗️ **Multi-Command Architecture**: Five main operation modes
  - `suix vanity` - Generate custom Sui vanity addresses  
  - `suix json-rpc` - Direct Sui JSON-RPC calls
  - `suix json-rpc-quick` - Quick access to common JSON-RPC methods
  - `suix grpc` - Raw gRPC calls (buf curl-like interface)
  - `suix grpc-quick` - Native gRPC client with real-time streaming

- 🚀 **High Performance**: Multi-threaded vanity address generation using Rayon
- 🎯 **Flexible Patterns**: Support for hexspeak conversion, hex patterns, and regex
- 🔐 **Official Sui Integration**: Uses `sui-keys`, `sui-types`, and `sui-rpc-api` for authentic operations
- 🌐 **Dual Protocol Support**: Both JSON-RPC and native gRPC protocols
- ⚡ **Real-time Streaming**: Live checkpoint subscription via gRPC
- 📊 **Pipeline Ready**: JSON output mode for automation and processing
- 🛠️ **Modern Rust**: Built with latest Rust ecosystem and workspace dependencies

## 📦 Installation

```bash
git clone https://github.com/longcipher/suix.git
cd suix
cargo build --release
```

The binary will be available at `./target/release/suix`.

## 🚀 Usage

### Main Commands

```bash
suix <COMMAND>

Commands:
  vanity          Generate Sui vanity addresses
  json-rpc        Make Sui JSON-RPC calls
  grpc            Make raw gRPC calls (buf curl-like interface)
  json-rpc-quick  Quick access to common JSON-RPC methods
  grpc-quick      Quick access to common gRPC methods (using sui-rpc-api)
  help            Print help information
```

## 💎 Vanity Address Generation

Generate custom Sui addresses with specific patterns.

### Basic Examples

```bash
# Generate address starting with "ace" (prints to terminal)
./suix vanity --starts-with ace

# Generate address ending with "beef"  
./suix vanity --ends-with beef

# Generate with both prefix and suffix
./suix vanity --starts-with bad --ends-with ace

# Save to file instead of printing to terminal
./suix vanity --starts-with cafe --save-path ./keys

# Generate multiple addresses with custom threads
./suix vanity --starts-with dead -n 5 -j 16
```

### Output Modes

**Terminal Output (Default):**

```bash
./suix vanity --starts-with abc
# Output:
# Found match 1/1:
# Address: 0xabc07e9245e1685ec5bb966a3137c0991d0109c654fc2a11e09a6b0c7f4b458d
# Private Key: APlI3fe6wAcjRTxTVKJBYKIORZOkWvmUnEi7F98o/1WG
```

**File Output:**

```bash
./suix vanity --starts-with def --save-path ./keys
# Output:
# Found match 1/1: 0xdef60fd... -> ./keys/def60fd....key
```

### Pattern Types

1. **Hexspeak Conversion** (default): Converts readable text to hex-like characters
   - `ace` → looks for bytes `[0xac, 0xe0]`
   - `cafe` → looks for bytes `[0xca, 0xfe]`

2. **Hex Strings**: Direct hex patterns
   - `0xabcd` → exact hex bytes `[0xab, 0xcd]`

3. **Regex Patterns**: Complex patterns with regex syntax
   - `^[a-f]{4}` → addresses starting with 4 hex letters

### Hexspeak Character Mapping

- `a,b,c,d,e,f` → `a,b,c,d,e,f` (unchanged)
- `g` → `9`, `i,j,l` → `1`, `o` → `0`, `q` → `9`
- `s` → `5`, `t` → `7`, `z` → `2`
- `0-9` → `0-9` (unchanged)

### Vanity Options

```bash
Options:
  --starts-with <PATTERN>        Prefix pattern
  --ends-with <PATTERN>          Suffix pattern  
  --save-path <PATH>             Save to file (optional)
  -j, --threads <THREADS>        Thread count [default: auto]
  -n, --count <COUNT>            Number of addresses [default: 1] 
  --addresses-per-round <COUNT>  Batch size [default: 10000]
```

## 🌐 Sui JSON-RPC Operations

Direct access to Sui blockchain via JSON-RPC.

### Basic RPC Calls

```bash
# Generic RPC call
./suix json-rpc <METHOD> [PARAMS] [OPTIONS]

# Get chain identifier
./suix json-rpc sui_getChainIdentifier

# Get latest checkpoint with pretty printing
./suix json-rpc sui_getLatestCheckpointSequenceNumber --pretty

# Get object information
./suix json-rpc sui_getObject '["0x123..."]' --pretty
```

### RPC Options

```bash
Options:
  --url <URL>      RPC endpoint [default: https://fullnode.mainnet.sui.io:443]
  -p, --pretty     Pretty print JSON response
```

## ⚡ Quick JSON-RPC Commands

Shortcuts for common blockchain queries via JSON-RPC.

```bash
# Get chain identifier
./suix json-rpc-quick chain [--pretty]

# Get latest checkpoint
./suix json-rpc-quick checkpoint [--pretty]

# Get object by ID
./suix json-rpc-quick object <OBJECT_ID> [--pretty]

# Get transaction by digest  
./suix json-rpc-quick tx <DIGEST> [--pretty]

# Get account balance
./suix json-rpc-quick balance <ADDRESS> [--pretty]
```

### JSON-RPC Quick Examples

```bash
# Quick chain info
./suix json-rpc-quick chain --pretty

# Latest checkpoint
./suix json-rpc-quick checkpoint

# Object details
./suix json-rpc-quick object 0x123... --pretty

# Account balance
./suix json-rpc-quick balance 0xabc... --pretty
```

## 🚀 Native gRPC Operations

High-performance native gRPC calls using sui-rpc-api client.

### Basic gRPC Calls

```bash
# Get service information (latest checkpoint)
./suix grpc-quick info [--pretty] [--json]

# Get object by ID
./suix grpc-quick object <OBJECT_ID> [--pretty] [--json]

# Get transaction by digest
./suix grpc-quick tx <DIGEST> [--pretty] [--json]

# Get full checkpoint data
./suix grpc-quick full-checkpoint <SEQUENCE> [--pretty] [--json]

# List available gRPC methods
./suix grpc-quick list-methods
```

### Real-time Streaming

```bash
# Subscribe to checkpoint stream (real-time)
./suix grpc-quick subscribe --stream [--interval 5] [--json]

# Example streaming with custom interval
./suix grpc-quick subscribe --stream --interval 3 --json

# Subscribe and save to file for processing
./suix grpc-quick subscribe --stream --json > checkpoints.jsonl
```

### Raw gRPC Interface

```bash
# Raw gRPC call (buf curl-like)
./suix grpc-quick curl <SERVICE> <METHOD> [DATA]

# Examples
./suix grpc-quick curl sui.rpc.v2beta2.LedgerService GetLatestCheckpoint
./suix grpc-quick curl sui.rpc.v2beta2.LedgerService GetCheckpoint '{"sequence_number": 12345}'
```

### gRPC Options

```bash
Options:
  --url <URL>           gRPC endpoint [default: https://fullnode.mainnet.sui.io:443]
  -p, --pretty          Pretty print the response
  -j, --json            Output only JSON for pipeline processing
  -s, --stream          Enable continuous streaming mode
  --interval <SECONDS>  Polling interval for streaming [default: 5]
  --timeout <SECONDS>   Request timeout [default: 30]
```

## 🏗️ Project Structure

```text
suix/
├── bin/suix/          # Main CLI application
├── crates/vanity/     # Vanity address generation
├── crates/rpc/        # JSON-RPC client functionality  
├── crates/grpc/       # Native gRPC client with streaming
└── Cargo.toml         # Workspace configuration
```

## 📚 Examples

### Gaming/Fun Addresses

```bash
# "game" themed
./suix vanity --starts-with 9a1e -n 3

# "cool" address
./suix vanity --starts-with c001 

# "leet" speak
./suix vanity --starts-with 1337
```

### Development Addresses

```bash
# "test" addresses
./suix vanity --starts-with 7e57 -n 10 --save-path ./test-keys

# "dev" addresses  
./suix vanity --starts-with de1 -n 5
```

### JSON-RPC Operations

```bash
# Monitor latest activity
./suix json-rpc-quick checkpoint --pretty

# Check specific transaction
./suix json-rpc-quick tx 0x123...abc --pretty

# Verify account balance
./suix json-rpc-quick balance 0xabc...def --pretty
```

### Native gRPC Operations

```bash
# Real-time checkpoint monitoring
./suix grpc-quick subscribe --stream --interval 3 --json

# Get detailed checkpoint data
./suix grpc-quick full-checkpoint 12345 --pretty

# Pipeline processing example
./suix grpc-quick subscribe --stream --json | jq '.sequence_number'

# Service connectivity test
./suix grpc-quick info --json
```

### Advanced Pipeline Examples

```bash
# Monitor new checkpoints and extract sequence numbers
./suix grpc-quick subscribe --stream --json | jq -r '.sequence_number'

# Get checkpoint info and format timestamp
./suix grpc-quick info --json | jq -r '.timestamp_ms | tonumber / 1000 | todate'

# Batch process multiple objects
echo '0x123 0x456 0x789' | xargs -n1 ./suix grpc-quick object --json
```

## 🔧 Performance Tips

1. **Optimal Threading**: Use `-j` equal to CPU cores for vanity generation
2. **Batch Tuning**: Increase `--addresses-per-round` for less frequent updates
3. **Pattern Complexity**: Simpler patterns generate faster
4. **Protocol Choice**: Use gRPC for better performance and real-time features
5. **Network Endpoints**: Use local/faster RPC endpoints for better response times
6. **Streaming Intervals**: Adjust `--interval` based on your monitoring needs
7. **Pipeline Processing**: Use `--json` flag for automated processing workflows

## 🌟 Key Features Explained

### Dual Protocol Support

Suix supports both JSON-RPC and native gRPC protocols:

- **JSON-RPC**: Traditional HTTP-based calls, compatible with all Sui nodes
- **gRPC**: High-performance binary protocol with streaming capabilities

### Live Streaming Capabilities

The gRPC client supports real-time checkpoint streaming:

```bash
# Continuous monitoring
./suix grpc-quick subscribe --stream --interval 5

# Pipeline-ready output
./suix grpc-quick subscribe --stream --json | jq '.'
```

### JSON Output Mode

gRPC commands support `--json` flag for pipeline processing:

```bash
# Machine-readable output  
./suix grpc-quick info --json
./suix grpc-quick subscribe --stream --json

# JSON-RPC commands output JSON by default (use --pretty for formatting)
./suix json-rpc-quick chain
./suix json-rpc-quick checkpoint --pretty
```

### Workspace Architecture

The project uses Cargo workspace for optimal dependency management:

- Shared dependencies across all crates
- Version inheritance from root workspace
- Feature-based compilation for minimal binary size

## 🛡️ Security & Safety

- ✅ Cryptographically secure key generation (Ed25519)
- ✅ Official Sui library integration
- ✅ No network communication for vanity generation
- ✅ Proper file permissions for saved keys
- ✅ Type-safe Rust implementation

## 🚀 Recent Updates

### v0.1.0 - Enhanced gRPC Support

- ✨ **Native gRPC Client**: Added `sui-rpc-api` integration for authentic gRPC calls
- 🔄 **Real-time Streaming**: Live checkpoint subscription with customizable intervals
- 📊 **Pipeline Support**: JSON output mode for automation workflows
- 🏗️ **Workspace Restructure**: Optimized Cargo workspace with proper dependency inheritance
- 🛠️ **Code Quality**: Full clippy compliance and modern Rust patterns
- 📡 **Dual Protocol**: Both JSON-RPC and gRPC support for maximum compatibility

### Protocol Comparison

| Feature | JSON-RPC | gRPC |
|---------|----------|------|
| Compatibility | ✅ Universal | ✅ Native Sui |
| Performance | ⚡ Good | ⚡⚡ Excellent |
| Streaming | ❌ No | ✅ Yes |
| Binary Size | 📦 Smaller | 📦 Larger |
| Use Case | Scripts/Tools | Real-time Apps |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

Licensed under the terms specified in the LICENSE file.

## ⚠️ Disclaimer

This tool is for educational and development purposes. Always verify generated addresses and secure private keys properly. The authors are not responsible for any loss of funds.
