use eyre::Result;
use serde_json::{Value, json};

pub mod tx;

/// Configuration for RPC client
#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub url: String,
    pub pretty: bool,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            url: "https://fullnode.mainnet.sui.io:443".to_string(),
            pretty: false,
        }
    }
}

pub const MAINNET_URL: &str = "https://fullnode.mainnet.sui.io:443";
pub const TESTNET_URL: &str = "https://fullnode.testnet.sui.io:443";
pub const DEVNET_URL: &str = "https://fullnode.devnet.sui.io:443";
pub const LOCALNET_URL: &str = "http://127.0.0.1:9000";

/// Resolve a network profile name to its default RPC URL.
pub fn profile_url(profile: &str) -> Option<&'static str> {
    match profile {
        "mainnet" => Some(MAINNET_URL),
        "testnet" => Some(TESTNET_URL),
        "devnet" => Some(DEVNET_URL),
        "localnet" | "local" => Some(LOCALNET_URL),
        _ => None,
    }
}

/// Make a JSON-RPC call to the Sui node and return the parsed response.
pub async fn rpc_call(url: &str, method: &str, params: Value) -> Result<Value> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| eyre::eyre!("HTTP request failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| eyre::eyre!("Failed to read response: {}", e))?;

    if !status.is_success() {
        eyre::bail!(
            "HTTP request failed with status {}: {}",
            status,
            response_text
        );
    }

    serde_json::from_str(&response_text).map_err(|e| eyre::eyre!("Invalid JSON response: {}", e))
}

/// Print a JSON-RPC response value, surfacing RPC-level errors.
pub fn print_response(response: &Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(response)?);
    } else {
        println!("{}", serde_json::to_string(response)?);
    }

    if let Some(error) = response.get("error") {
        eprintln!("RPC Error: {}", serde_json::to_string_pretty(error)?);
    }

    Ok(())
}

/// Make a JSON-RPC call to the Sui node
pub async fn make_rpc_call(config: &RpcConfig, method: &str, params: Option<&str>) -> Result<()> {
    let params_value: Value = if let Some(params_str) = params {
        serde_json::from_str(params_str)
            .map_err(|e| eyre::eyre!("Invalid JSON parameters: {}", e))?
    } else {
        json!([])
    };

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params_value
    });

    if config.pretty {
        println!("Making RPC call to: {}", config.url);
        println!("Method: {method}");
        println!("Request:");
        println!("{}", serde_json::to_string_pretty(&request)?);
        println!();
    }

    let response_json: Value = rpc_call(&config.url, method, params_value).await?;

    println!("Response:");
    print_response(&response_json, config.pretty)
}

/// Common Sui RPC methods with helper functions
pub mod methods {
    use super::*;

    /// Get the chain identifier
    pub async fn get_chain_identifier(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "sui_getChainIdentifier", None).await
    }

    /// Get the latest checkpoint sequence number
    pub async fn get_latest_checkpoint_sequence_number(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "sui_getLatestCheckpointSequenceNumber", None).await
    }

    /// Get object information by ID
    pub async fn get_object(config: &RpcConfig, object_id: &str) -> Result<()> {
        let params = format!(
            r#"["{object_id}", {{"showType": true, "showOwner": true, "showPreviousTransaction": true, "showDisplay": false, "showContent": true, "showBcs": false, "showStorageRebate": true}}]"#
        );
        make_rpc_call(config, "sui_getObject", Some(&params)).await
    }

    /// Get transaction by digest
    pub async fn get_transaction_block(config: &RpcConfig, digest: &str) -> Result<()> {
        let params = format!(
            r#"["{digest}", {{"showInput": true, "showRawInput": false, "showEffects": true, "showEvents": true, "showObjectChanges": true, "showBalanceChanges": true}}]"#
        );
        make_rpc_call(config, "sui_getTransactionBlock", Some(&params)).await
    }

    /// Get account balance
    pub async fn get_balance(
        config: &RpcConfig,
        address: &str,
        coin_type: Option<&str>,
    ) -> Result<()> {
        let params = if let Some(coin) = coin_type {
            format!(r#"["{address}", "{coin}"]"#)
        } else {
            format!(r#"["{address}"]"#)
        };
        make_rpc_call(config, "suix_getBalance", Some(&params)).await
    }

    /// Get all balances of an address
    pub async fn get_all_balances(config: &RpcConfig, address: &str) -> Result<()> {
        let params = format!(r#"["{address}"]"#);
        make_rpc_call(config, "suix_getAllBalances", Some(&params)).await
    }

    /// Get coins owned by an address
    pub async fn get_coins(
        config: &RpcConfig,
        address: &str,
        coin_type: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<()> {
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = if let Some(coin) = coin_type {
            format!(
                r#"["{address}", "{coin}", {cursor_json}, {}]"#,
                limit.unwrap_or(20)
            )
        } else {
            format!(
                r#"["{address}", null, {cursor_json}, {}]"#,
                limit.unwrap_or(20)
            )
        };
        make_rpc_call(config, "suix_getCoins", Some(&params)).await
    }

    /// Get all coins owned by an address
    pub async fn get_all_coins(
        config: &RpcConfig,
        address: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<()> {
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = format!(r#"["{address}", {cursor_json}, {}]"#, limit.unwrap_or(20));
        make_rpc_call(config, "suix_getAllCoins", Some(&params)).await
    }

    /// Get objects owned by an address
    pub async fn get_owned_objects(
        config: &RpcConfig,
        address: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<()> {
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = format!(
            r#"["{address}", {{"filter": null, "options": {{"showType": true, "showOwner": true, "showContent": true}}}}, {cursor_json}, {}]"#,
            limit.unwrap_or(20)
        );
        make_rpc_call(config, "sui_getOwnedObjects", Some(&params)).await
    }

    /// Get dynamic fields of a parent object
    pub async fn get_dynamic_fields(
        config: &RpcConfig,
        parent_id: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<()> {
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = format!(r#"["{parent_id}", {cursor_json}, {}]"#, limit.unwrap_or(20));
        make_rpc_call(config, "sui_getDynamicFields", Some(&params)).await
    }

    /// Query events with a filter
    pub async fn query_events(
        config: &RpcConfig,
        filter_json: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<()> {
        let filter = filter_json.unwrap_or("{\"All\": []}");
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = format!(
            r#"[{filter}, {cursor_json}, {}, false]"#,
            limit.unwrap_or(20)
        );
        make_rpc_call(config, "suix_queryEvents", Some(&params)).await
    }

    /// Get staked SUI for an owner
    pub async fn get_stakes(config: &RpcConfig, owner: &str) -> Result<()> {
        let params = format!(r#"["{owner}"]"#);
        make_rpc_call(config, "suix_getStakes", Some(&params)).await
    }

    /// Get reference gas price
    pub async fn get_reference_gas_price(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "sui_getReferenceGasPrice", None).await
    }

    /// Get current epoch info
    pub async fn get_current_epoch(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "suix_getCurrentEpoch", None).await
    }

    /// Get committee info for an epoch
    pub async fn get_committee_info(config: &RpcConfig, epoch: Option<u64>) -> Result<()> {
        let params = epoch.map_or("[]".to_string(), |e| format!(r#"["{e}"]"#));
        make_rpc_call(config, "sui_getCommitteeInfo", Some(&params)).await
    }

    /// Get latest SUI system state
    pub async fn get_latest_system_state(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "sui_getLatestSuiSystemState", None).await
    }

    /// Get validators APY table
    pub async fn get_validators_apy(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "suix_getValidatorsApy", None).await
    }

    /// Get coin metadata
    pub async fn get_coin_metadata(config: &RpcConfig, coin_type: &str) -> Result<()> {
        let params = format!(r#"["{coin_type}"]"#);
        make_rpc_call(config, "sui_getCoinMetadata", Some(&params)).await
    }

    /// Get total coin supply
    pub async fn get_total_supply(config: &RpcConfig, coin_type: &str) -> Result<()> {
        let params = format!(r#"["{coin_type}"]"#);
        make_rpc_call(config, "sui_getTotalSupply", Some(&params)).await
    }

    /// Get total transaction count
    pub async fn get_total_transactions(config: &RpcConfig) -> Result<()> {
        make_rpc_call(config, "sui_getTotalTransactionBlocks", None).await
    }

    /// Get a checkpoint by sequence number or digest
    pub async fn get_checkpoint(config: &RpcConfig, id: &str) -> Result<()> {
        let params = format!(r#"["{id}"]"#);
        make_rpc_call(config, "sui_getCheckpoint", Some(&params)).await
    }

    /// Get a page of checkpoints
    pub async fn get_checkpoints(
        config: &RpcConfig,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<()> {
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = format!(r#"[{cursor_json}, {}, false]"#, limit.unwrap_or(20));
        make_rpc_call(config, "sui_getCheckpoints", Some(&params)).await
    }

    /// Query transaction blocks with a filter
    pub async fn query_transactions(
        config: &RpcConfig,
        filter_json: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<()> {
        let filter = filter_json.unwrap_or("{\"FromAddress\": \"0x0\"}");
        let cursor_json = cursor.map_or("null".to_string(), |c| format!(r#""{c}""#));
        let params = format!(
            r#"[{{"filter": {filter}, "options": {{"showInput": true, "showEffects": true}}}}, {cursor_json}, {}, false]"#,
            limit.unwrap_or(20)
        );
        make_rpc_call(config, "sui_queryTransactionBlocks", Some(&params)).await
    }

    /// Try to get a past object version
    pub async fn try_get_past_object(
        config: &RpcConfig,
        object_id: &str,
        version: u64,
    ) -> Result<()> {
        let params = format!(
            r#"["{object_id}", {version}, {{"showType": true, "showOwner": true, "showContent": true}}]"#
        );
        make_rpc_call(config, "sui_tryGetPastObject", Some(&params)).await
    }

    /// Get normalized Move modules of a package
    pub async fn get_normalized_move_modules(config: &RpcConfig, package_id: &str) -> Result<()> {
        let params = format!(r#"["{package_id}"]"#);
        make_rpc_call(
            config,
            "sui_getNormalizedMoveModulesByPackage",
            Some(&params),
        )
        .await
    }

    /// Get Move function argument types
    pub async fn get_move_function_arg_types(
        config: &RpcConfig,
        package_id: &str,
        module: &str,
        function: &str,
    ) -> Result<()> {
        let params = format!(r#"["{package_id}", "{module}", "{function}"]"#);
        make_rpc_call(config, "sui_getMoveFunctionArgTypes", Some(&params)).await
    }

    /// Get protocol config
    pub async fn get_protocol_config(config: &RpcConfig, version: Option<u64>) -> Result<()> {
        let params = version.map_or("[]".to_string(), |v| format!(r#"["{v}"]"#));
        make_rpc_call(config, "sui_getProtocolConfig", Some(&params)).await
    }

    /// Dry-run an unsigned transaction block
    pub async fn dry_run(config: &RpcConfig, tx_bytes: &str) -> Result<()> {
        let params = format!(r#"["{tx_bytes}"]"#);
        make_rpc_call(config, "sui_dryRunTransactionBlock", Some(&params)).await
    }

    /// Dev-inspect a move call without executing
    pub async fn dev_inspect(
        config: &RpcConfig,
        sender: &str,
        package_id: &str,
        module: &str,
        function: &str,
        type_args_json: &str,
        args_json: &str,
    ) -> Result<()> {
        let params = format!(
            r#"["{sender}", "{package_id}", "{module}", "{function}", {type_args_json}, {args_json}, null, null]"#
        );
        make_rpc_call(config, "sui_devInspectTransactionBlock", Some(&params)).await
    }

    /// Execute a signed transaction block
    pub async fn execute_transaction(
        config: &RpcConfig,
        tx_bytes: &str,
        signatures_json: &str,
    ) -> Result<()> {
        let params = format!(
            r#"["{tx_bytes}", {signatures_json}, {{"showInput": true, "showEffects": true, "showEvents": true}}, "WaitForLocalExecution"]"#
        );
        make_rpc_call(config, "sui_executeTransactionBlock", Some(&params)).await
    }

    /// Resolve a SuiNS name to an address
    pub async fn resolve_name(config: &RpcConfig, name: &str) -> Result<()> {
        let params = format!(r#"["{name}"]"#);
        make_rpc_call(config, "suix_resolveNameServiceAddress", Some(&params)).await
    }

    /// Reverse-resolve an address to SuiNS names
    pub async fn reverse_resolve(config: &RpcConfig, address: &str) -> Result<()> {
        let params = format!(r#"["{address}"]"#);
        make_rpc_call(config, "suix_resolveNameServiceNames", Some(&params)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_config_default() {
        let config = RpcConfig::default();
        assert_eq!(config.url, "https://fullnode.mainnet.sui.io:443");
        assert!(!config.pretty);
    }

    #[test]
    fn test_profile_url() {
        assert_eq!(profile_url("mainnet"), Some(MAINNET_URL));
        assert_eq!(profile_url("testnet"), Some(TESTNET_URL));
        assert_eq!(profile_url("devnet"), Some(DEVNET_URL));
        assert_eq!(profile_url("localnet"), Some(LOCALNET_URL));
        assert_eq!(profile_url("local"), Some(LOCALNET_URL));
        assert_eq!(profile_url("unknown"), None);
    }
}
