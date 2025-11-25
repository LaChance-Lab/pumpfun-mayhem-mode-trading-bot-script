# Monad Trading Bots(Rust)
A comprehensive Rust SDK for interacting with Nad.fun ecosystem contracts, including bonding curves, DEX trading, and real-time event monitoring.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nadfun_sdk = "0.2.1"
```

## Quick Start

```rust
use nadfun_sdk::prelude::*; // Import everything you need
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://your-rpc-endpoint".to_string();
    let private_key = "your_private_key_here".to_string();

    // Trading with new gas estimation system
    let trade = Trade::new(rpc_url.clone(), private_key.clone()).await?;
    let token: Address = "0x...".parse()?;
    let (router, amount_out) = trade.get_amount_out(token, parse_ether("0.1")?, true).await?;

    // New unified gas estimation (v0.2.0)
    let gas_params = GasEstimationParams::Buy {
        token,
        amount_in: parse_ether("0.1")?,
        amount_out_min: amount_out,
        to: trade.wallet_address(),
        deadline: U256::from(9999999999999999u64),
    };
    let estimated_gas = trade.estimate_gas(&router, gas_params).await?;

    // Token operations
    let token_helper = TokenHelper::new(rpc_url, private_key).await?;
    let balance = token_helper.balance_of(token, "0x...".parse()?).await?;

    Ok(())
}
```

## Features

### 🚀 Trading

Execute buy/sell operations on bonding curves with slippage protection:

```rust
use nadfun_sdk::{Trade, SlippageUtils, GasEstimationParams, types::BuyParams};

// Get quote and execute buy
let (router, expected_tokens) = trade.get_amount_out(token, mon_amount, true).await?;
let min_tokens = SlippageUtils::calculate_amount_out_min(expected_tokens, 5.0);

// Use new unified gas estimation system
let gas_params = GasEstimationParams::Buy {
    token,
    amount_in: mon_amount,
    amount_out_min: min_tokens,
    to: wallet_address,
    deadline: U256::from(deadline),
};

// Get accurate gas estimation from network
let estimated_gas = trade.estimate_gas(&router, gas_params).await?;
let gas_with_buffer = estimated_gas * 120 / 100; // Add 20% buffer

let buy_params = BuyParams {
    token,
    amount_in: mon_amount,
    amount_out_min: min_tokens,
    to: wallet_address,
    deadline: U256::from(deadline),
    gas_limit: Some(gas_with_buffer), // Use network-based estimation
    gas_price: Some(50_000_000_000), // 50 gwei
    nonce: None, // Auto-detect
};

let result = trade.buy(buy_params, router).await?;
```

### ⛽ Gas Management

**v0.2.0 introduces a unified gas estimation system** that replaces static constants with real-time network estimation:

#### Unified Gas Estimation (New in v0.2.0)

```rust
use nadfun_sdk::{GasEstimationParams, Trade};

// Create gas estimation parameters for any operation
let gas_params = GasEstimationParams::Buy {
    token,
    amount_in: mon_amount,
    amount_out_min: min_tokens,
    to: wallet_address,
    deadline: U256::from(deadline),
};

// Get real-time gas estimation from network
let estimated_gas = trade.estimate_gas(&router, gas_params).await?;

// Apply buffer strategy
let gas_with_buffer = estimated_gas * 120 / 100; // 20% buffer
```

#### Gas Estimation Parameters

```rust
pub enum GasEstimationParams {
    // For buying tokens
    Buy { token, amount_in, amount_out_min, to, deadline },

    // For selling tokens (requires token approval)
    Sell { token, amount_in, amount_out_min, to, deadline },

    // For gasless selling with permits
    SellPermit { token, amount_in, amount_out_min, to, deadline, v, r, s },
}
```

#### Automatic Problem Solving

The new system automatically handles common issues:

- **Token Approval**: SELL operations automatically check and approve tokens
- **Permit Signatures**: SELL PERMIT operations generate real EIP-2612 signatures
- **Network Conditions**: Uses actual network state instead of static estimates
- **Error Recovery**: Graceful fallback when estimation fails

#### Buffer Strategies

```rust
// Fixed buffer amounts
let gas_fixed_buffer = estimated_gas + 50_000;  // +50k gas

// Percentage-based buffers
let gas_20_percent = estimated_gas * 120 / 100; // 20% buffer
let gas_25_percent = estimated_gas * 125 / 100; // 25% buffer (for complex operations)

// Choose based on operation complexity
let final_gas = match operation_type {
    "buy" => estimated_gas * 120 / 100,        // 20% buffer
    "sell" => estimated_gas * 115 / 100,       // 15% buffer
    "sell_permit" => estimated_gas * 125 / 100, // 25% buffer
    _ => estimated_gas + 50_000,               // Fixed buffer
};
```

#### Migration from v0.1.x

```rust
// OLD (v0.1.x) - Static constants
use nadfun_sdk::{BondingCurveGas, get_default_gas_limit, Operation};
let gas_limit = get_default_gas_limit(&router, Operation::Buy);

// NEW (v0.2.0) - Network-based estimation
use nadfun_sdk::GasEstimationParams;
let params = GasEstimationParams::Buy { token, amount_in, amount_out_min, to, deadline };
let estimated_gas = trade.estimate_gas(&router, params).await?;
let gas_limit = estimated_gas * 120 / 100; // Apply buffer
```

**⚠️ Important Notes:**

- **SELL Operations**: Require token approval for router (automatically handled in examples)
- **SELL PERMIT Operations**: Need valid EIP-2612 permit signatures (automatically generated)
- **Network Connection**: Live RPC required for accurate estimation

### 📊 Token Operations

Interact with ERC-20 tokens and get metadata:

```rust
use nadfun_sdk::TokenHelper;

let token_helper = TokenHelper::new(rpc_url, private_key).await?;

// Get token metadata
let metadata = token_helper.get_token_metadata(token).await?;
println!("Token: {} ({})", metadata.name, metadata.symbol);

// Check balances and allowances
let balance = token_helper.balance_of(token, wallet).await?;
let allowance = token_helper.allowance(token, owner, spender).await?;

// Approve tokens
let tx = token_helper.approve(token, spender, amount).await?;
```

### 🔄 Real-time Event Streaming

Monitor bonding curve and DEX events in real-time:

#### Bonding Curve Streaming

```rust
use nadfun_sdk::stream::CurveStream;
use nadfun_sdk::types::{BondingCurveEvent, EventType};
use futures_util::{pin_mut, StreamExt};

// Create WebSocket stream
let curve_stream = CurveStream::new("wss://your-ws-endpoint".to_string()).await?;

// Configure filters (optional)
let curve_stream = curve_stream
    .subscribe_events(vec![EventType::Buy, EventType::Sell])
    .filter_tokens(vec![token_address]);

// Subscribe and process events
let stream = curve_stream.subscribe().await?;
pin_mut!(stream);

while let Some(event_result) = stream.next().await {
    match event_result {
        Ok(event) => {
            println!("Event: {:?} for token {}", event.event_type(), event.token());
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

#### DEX Swap Streaming

```rust
use nadfun_sdk::stream::UniswapSwapStream;
use futures_util::{pin_mut, StreamExt};

// Auto-discover pools for tokens
let swap_stream = UniswapSwapStream::discover_pools_for_tokens(
    "wss://your-ws-endpoint".to_string(),
    vec![token_address]
).await?;

// Subscribe and process events
let stream = swap_stream.subscribe().await?;
pin_mut!(stream);

while let Some(event_result) = stream.next().await {
    match event_result {
        Ok(event) => {
            println!("Swap in pool {}: {} -> {}",
                event.pool_address, event.amount0, event.amount1);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

### 📈 Historical Data Analysis

Fetch and analyze historical events:

```rust
use nadfun_sdk::stream::{CurveIndexer, EventType};

let provider = Arc::new(ProviderBuilder::new().connect_http(http_url.parse()?));
let indexer = CurveIndexer::new(provider);

// Fetch events from block range
let events = indexer.fetch_events(
    18_000_000,
    18_010_000,
    vec![EventType::Create, EventType::Buy],
    None // all tokens
).await?;

println!("Found {} events", events.len());
```

### 🔍 Pool Discovery

Find Uniswap V3 pool addresses for tokens:

```rust
use nadfun_sdk::stream::UniswapSwapIndexer;

// Auto-discover pools for multiple tokens
let indexer = UniswapSwapIndexer::discover_pools_for_tokens(provider, tokens).await?;
let pools = indexer.pool_addresses();

// Discover pool for single token
let indexer = UniswapSwapIndexer::discover_pool_for_token(provider, token).await?;
```

### 💱 DEX Monitoring

Monitor Uniswap V3 swap events:

```rust
use nadfun_sdk::stream::UniswapSwapIndexer;

// Auto-discover pools for tokens
let indexer = UniswapSwapIndexer::discover_pools_for_tokens(provider, tokens).await?;
let swaps = indexer.fetch_events(from_block, to_block).await?;

for swap in swaps {
    println!("Swap in pool {}: {} -> {}",
        swap.pool_metadata.pool_address,
        swap.amount0,
        swap.amount1
    );
}
```

## Examples

The SDK includes comprehensive examples in the `examples/` directory:

### Trading Examples

```bash
# Using environment variables
export PRIVATE_KEY="your_private_key_here"
export RPC_URL="https://your-rpc-endpoint"
export TOKEN="0xTokenAddress"
export RECIPIENT="0xRecipientAddress"  # For token operations

cargo run --example buy              # Buy tokens with network-based gas estimation
cargo run --example sell             # Sell tokens with automatic approval handling
cargo run --example sell_permit      # Gasless sell with real permit signatures
cargo run --example gas_estimation   # Comprehensive gas estimation example (NEW)
cargo run --example basic_operations # Token operations (requires recipient)

# Using command line arguments
cargo run --example buy -- --private-key your_private_key_here --rpc-url https://your-rpc-endpoint --token 0xTokenAddress
cargo run --example sell -- --private-key your_private_key_here --rpc-url https://your-rpc-endpoint --token 0xTokenAddress
cargo run --example sell_permit -- --private-key your_private_key_here --rpc-url https://your-rpc-endpoint --token 0xTokenAddress
cargo run --example gas_estimation -- --private-key your_private_key_here --rpc-url https://your-rpc-endpoint --token 0xTokenAddress
cargo run --example basic_operations -- --private-key your_private_key_here --rpc-url https://your-rpc-endpoint --token 0xTokenAddress --recipient 0xRecipientAddress
```

### Gas Estimation Example (New in v0.2.0)

```bash
# Comprehensive gas estimation with automatic problem solving
cargo run --example gas_estimation -- --private-key your_private_key_here --rpc-url https://your-rpc-endpoint --token 0xTokenAddress
```

**Features:**

- **Unified Gas Estimation**: Demonstrates `trade.estimate_gas()` for all operation types
- **Automatic Approval**: Handles token approval for SELL operations automatically
- **Real Permit Signatures**: Generates valid EIP-2612 signatures for SELL PERMIT operations
- **Buffer Strategies**: Shows different buffer calculation methods (fixed +50k, percentage 20%-25%)
- **Cost Analysis**: Real-time transaction cost estimates at different gas prices
- **Error Handling**: Graceful fallback when estimation fails

### Token Examples

```bash
cargo run --example basic_operations # Basic ERC-20 operations
cargo run --example permit_signature # EIP-2612 permit signatures
```

### Stream Examples

The SDK provides 5 comprehensive streaming examples organized by category:

#### 🔄 Bonding Curve Examples

**1. curve_indexer** - Historical bonding curve event analysis

```bash
# Fetch historical Create, Buy, Sell events
cargo run --example curve_indexer -- --rpc-url https://your-rpc-endpoint

# With specific tokens and time range
cargo run --example curve_indexer -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xToken1,0xToken2
```

**2. curve_stream** - Real-time bonding curve monitoring

```bash
# Scenario 1: Monitor all bonding curve events
cargo run --example curve_stream -- --ws-url wss://your-ws-endpoint

# Scenario 2: Filter specific event types (Buy/Sell only)
EVENTS=Buy,Sell cargo run --example curve_stream -- --ws-url wss://your-ws-endpoint

# Scenario 3: Filter specific tokens only
cargo run --example curve_stream -- \
  --ws-url wss://your-ws-endpoint \
  --tokens 0xToken1,0xToken2

# Scenario 4: Combined filtering (events AND tokens)
EVENTS=Buy,Sell cargo run --example curve_stream -- \
  --ws-url wss://your-ws-endpoint \
  --tokens 0xToken1
```

**Features:**

- ✅ All event types: Create, Buy, Sell, Sync, Lock, Listed
- ✅ Event type filtering via `EVENTS` environment variable
- ✅ Token filtering via `--tokens` argument
- ✅ Combined filtering (events + tokens)
- ✅ Real-time WebSocket streaming
- ✅ Automatic event decoding

#### 💱 DEX Examples

**3. dex_indexer** - Historical DEX swap data analysis

```bash
# Discover pools and fetch historical swap events
cargo run --example dex_indexer -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xToken1,0xToken2

# Batch process with JSON array format
cargo run --example dex_indexer -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens '["0xToken1","0xToken2"]'
```

**4. dex_stream** - Real-time DEX swap monitoring

```bash
# Scenario 1: Monitor specific pool addresses directly
POOLS=0xPool1,0xPool2 cargo run --example dex_stream -- --ws-url wss://your-ws-endpoint

# Scenario 2: Auto-discover pools for multiple tokens
cargo run --example dex_stream -- \
  --ws-url wss://your-ws-endpoint \
  --tokens 0xToken1,0xToken2

# Scenario 3: Single token pool discovery
cargo run --example dex_stream -- \
  --ws-url wss://your-ws-endpoint \
  --token 0xTokenAddress
```

**Features:**

- ✅ Automatic pool discovery for tokens
- ✅ Direct pool address monitoring
- ✅ Single token pool discovery
- ✅ Real-time Uniswap V3 swap events
- ✅ Pool metadata included
- ✅ WebSocket streaming

#### 🔍 Pool Discovery

**5. pool_discovery** - Automated pool address discovery

```bash
# Find Uniswap V3 pools for multiple tokens
cargo run --example pool_discovery -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xToken1,0xToken2

# Discover pools for single token
cargo run --example pool_discovery -- \
  --rpc-url https://your-rpc-endpoint \
  --token 0xTokenAddress
```

### Testing & Verification

All examples have been tested and verified working. Here are ready-to-run test commands:

#### 🔄 Real-time Streaming Tests

```bash
# Test bonding curve streaming (all events)
cargo run --example curve_stream -- --ws-url wss://your-ws-endpoint

# Test DEX swap streaming (auto-discover pools)
cargo run --example dex_stream -- \
  --ws-url wss://your-ws-endpoint \
  --tokens 0xYourTokenAddress

# Test with event filtering
EVENTS=Buy,Sell cargo run --example curve_stream -- --ws-url wss://your-ws-endpoint

# Test with specific pool monitoring
POOLS=0xPool1,0xPool2 cargo run --example dex_stream -- --ws-url wss://your-ws-endpoint
```

#### 📊 Historical Data Tests

```bash
# Test bonding curve historical analysis
cargo run --example curve_indexer -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xYourTokenAddress

# Test pool discovery
cargo run --example pool_discovery -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xToken1,0xToken2

# Test DEX historical analysis
cargo run --example dex_indexer -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xYourTokenAddress
```

#### ⚡ Quick Validation

```bash
# Minimal test - just connect and verify
cargo run --example curve_stream -- --ws-url wss://your-ws-endpoint
# Should output: "Listening for ALL bonding curve events..."

cargo run --example dex_stream -- --token 0xTokenAddress --ws-url wss://your-ws-endpoint
# Should output: "Discovered X pools for 1 tokens"
```

## Core Types

### Event Types

- `BondingCurveEvent`: Unified enum for all bonding curve events
  - `Create`, `Buy`, `Sell`, `Sync`, `Lock`, `Listed` variants
  - Methods: `.token()`, `.event_type()`, `.block_number()`, `.transaction_index()`
- `SwapEvent`: Uniswap V3 swap events with complete metadata
  - Fields: `pool_address`, `amount0`, `amount1`, `sender`, `recipient`, `liquidity`, `tick`, `sqrt_price_x96`
- `EventType`: Enum for filtering bonding curve events
  - Variants: `Create`, `Buy`, `Sell`, `Sync`, `Lock`, `Listed`

### Stream Types

- `CurveStream`: Bonding curve event streaming
  - Methods: `.subscribe_events()`, `.filter_tokens()`, `.subscribe()`
  - Returns: `Pin<Box<dyn Stream<Item = Result<BondingCurveEvent>> + Send>>`
- `UniswapSwapStream`: DEX swap event streaming
  - Methods: `.new()`, `.discover_pools_for_tokens()`, `.discover_pool_for_token()`, `.subscribe()`
  - Returns: `Pin<Box<dyn Stream<Item = Result<SwapEvent>> + Send>>`

### Trading Types

- `BuyParams` / `SellParams`: Parameters for buy/sell operations
- `TradeResult`: Transaction result with status and metadata
- `SlippageUtils`: Utilities for slippage calculations

### Token Types

- `TokenMetadata`: Name, symbol, decimals, total supply
- `PermitSignature`: EIP-2612 permit signature data

## Configuration

### Environment Variables

```bash
export RPC_URL="https://your-rpc-endpoint"
export PRIVATE_KEY="your_private_key_here"
export WS_URL="wss://your-ws-endpoint"
export TOKEN="0xTokenAddress"
export TOKENS="0xToken1,0xToken2"  # Multiple tokens for monitoring
export RECIPIENT="0xRecipientAddress"
```

### CLI Arguments

All examples support command line arguments for configuration:

```bash
# Available options
--rpc-url <URL>      # RPC URL (default: https://your-rpc-endpoint)
--ws-url <URL>       # WebSocket URL (default: wss://your-ws-endpoint)
--private-key <KEY>  # Private key for transactions
--token <ADDRESS>    # Token address for operations
--tokens <ADDRS>     # Token addresses: 'addr1,addr2' or '["addr1","addr2"]'
--recipient <ADDR>   # Recipient address for transfers/allowances
--help, -h           # Show help

# Example usage
cargo run --example sell_permit -- \
  --rpc-url https://your-rpc-endpoint \
  --private-key your_private_key_here \
  --token 0xYourTokenAddress

# Example with recipient (for token operations)
cargo run --example basic_operations -- \
  --private-key your_private_key_here \
  --rpc-url https://your-rpc-endpoint \
  --token 0xYourTokenAddress \
  --recipient 0xRecipientAddress

# Example with multiple tokens for monitoring
cargo run --example dex_indexer -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens 0xToken1,0xToken2,0xToken3

# Example with JSON array format
cargo run --example pool_discovery -- \
  --rpc-url https://your-rpc-endpoint \
  --tokens '["0xToken1","0xToken2"]'
```

### Contract Addresses

All contract addresses are defined in `constants.rs`:

- Bonding Curve: `0x52D34d8536350Cd997bCBD0b9E9d722452f341F5`
- Bonding Curve Router: `0x4F5A3518F082275edf59026f72B66AC2838c0414`
- DEX Router: `0x4FBDC27FAE5f99E7B09590bEc8Bf20481FCf9551`
- WMON Token: `0x760AfE86e5de5fa0Ee542fc7B7B713e1c5425701`

## Error Handling

The SDK uses `anyhow::Result` for error handling:

```rust
use anyhow::Result;

async fn example() -> Result<()> {
    let trade = Trade::new(rpc_url, private_key).await?;
    let result = trade.get_amount_out(token, amount, true).await?;
    Ok(())
}
```

## Performance & Reliability

### ✅ Verified Features

- **Real-time Streaming**: WebSocket-based event delivery tested and working
- **Event Decoding**: Automatic parsing of bonding curve and swap events
- **Connection Stability**: Streams remain alive and process events continuously
- **Error Handling**: Graceful error handling with `Result<Event>` pattern
- **Multiple Scenarios**: All streaming scenarios tested and verified

### 📊 Tested Scenarios

- **Bonding Curve**: 4 scenarios (all events, filtered events, filtered tokens, combined)
- **DEX Streaming**: 3 scenarios (specific pools, token discovery, single token)
- **Historical Data**: Block range processing with automatic batching
- **Pool Discovery**: Automatic Uniswap V3 pool detection for tokens

### ⚡ Performance Features

- **Efficient Filtering**: Network-level filtering for event types
- **Client-side Filtering**: Token-based filtering for precise control
- **Concurrent Processing**: Parallel block processing for historical data
- **Memory Efficient**: Stream-based processing without buffering

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request
