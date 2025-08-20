# Suix - Comprehensive Sui CLI Tool

A high-performance, multi-purpose CLI tool for Sui blockchain operations, including vanity address generation and JSON-RPC interactions.

## ✨ Features

- 🏗️ **Multi-Command Architecture**: Three main operation modes
  - `suix vanity` - Generate custom Sui vanity addresses  
  - `suix rpc` - Direct Sui JSON-RPC calls
  - `suix query` - Quick access to common blockchain queries

- 🚀 **High Performance**: Multi-threaded vanity address generation using Rayon
- 🎯 **Flexible Patterns**: Support for hexspeak conversion, hex patterns, and regex
- 🔐 **Official Sui Integration**: Uses `sui-keys` and `sui-types` for authentic address generation
- 🌐 **Blockchain Operations**: Complete RPC client for Sui network interactions
- ⚡ **Modern Rust**: Built with latest Rust ecosystem and workspace dependencies

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
  vanity  Generate Sui vanity addresses
  rpc     Make Sui JSON-RPC calls  
  query   Quick access to common RPC methods
  help    Print help information
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
./suix rpc <METHOD> [PARAMS] [OPTIONS]

# Get chain identifier
./suix rpc sui_getChainIdentifier

# Get latest checkpoint with pretty printing
./suix rpc sui_getLatestCheckpointSequenceNumber --pretty

# Get object information
./suix rpc sui_getObject '["0x123..."]' --pretty
```

### RPC Options

```bash
Options:
  --url <URL>      RPC endpoint [default: https://fullnode.mainnet.sui.io:443]
  -p, --pretty     Pretty print JSON response
```

## ⚡ Quick Query Commands

Shortcuts for common blockchain queries.

```bash
# Get chain identifier
./suix query chain [--pretty]

# Get latest checkpoint
./suix query checkpoint [--pretty]

# Get object by ID
./suix query object <OBJECT_ID> [--pretty]

# Get transaction by digest  
./suix query tx <DIGEST> [--pretty]

# Get account balance
./suix query balance <ADDRESS> [--coin-type <TYPE>] [--pretty]
```

### Query Examples

```bash
# Quick chain info
./suix query chain --pretty

# Latest checkpoint
./suix query checkpoint

# Object details
./suix query object 0x123... --pretty

# Account balance
./suix query balance 0xabc... --pretty

# Specific coin balance
./suix query balance 0xdef... --coin-type "0x2::sui::SUI"
```

## 🏗️ Project Structure

```text
suix/
├── bin/suix/          # Main CLI application
├── crates/vanity/     # Vanity address generation
├── crates/rpc/        # RPC client functionality  
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

### Blockchain Operations

```bash
# Monitor latest activity
./suix query checkpoint --pretty

# Check specific transaction
./suix query tx 0x123...abc --pretty

# Verify account balance
./suix query balance 0xabc...def --pretty
```

## 🔧 Performance Tips

1. **Optimal Threading**: Use `-j` equal to CPU cores for vanity generation
2. **Batch Tuning**: Increase `--addresses-per-round` for less frequent updates
3. **Pattern Complexity**: Simpler patterns generate faster
4. **Network Endpoints**: Use local/faster RPC endpoints for better response times

## 🛡️ Security & Safety

- ✅ Cryptographically secure key generation (Ed25519)
- ✅ Official Sui library integration
- ✅ No network communication for vanity generation
- ✅ Proper file permissions for saved keys
- ✅ Type-safe Rust implementation

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

Licensed under the terms specified in the LICENSE file.

## ⚠️ Disclaimer

This tool is for educational and development purposes. Always verify generated addresses and secure private keys properly. The authors are not responsible for any loss of funds.
