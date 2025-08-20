use eyre::Result;
use serde_json::{Value, json};

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

/// Make a JSON-RPC call to the Sui node
pub async fn make_rpc_call(config: &RpcConfig, method: &str, params: Option<&str>) -> Result<()> {
    // Parse parameters if provided
    let params_value: Value = if let Some(params_str) = params {
        serde_json::from_str(params_str)
            .map_err(|e| eyre::eyre!("Invalid JSON parameters: {}", e))?
    } else {
        json!([])
    };

    // Construct JSON-RPC request
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

    // Make the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post(&config.url)
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

    // Parse and display the response
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| eyre::eyre!("Invalid JSON response: {}", e))?;

    if config.pretty {
        println!("Response:");
        println!("{}", serde_json::to_string_pretty(&response_json)?);
    } else {
        println!("{}", serde_json::to_string(&response_json)?);
    }

    // Check for JSON-RPC errors
    if let Some(error) = response_json.get("error") {
        eprintln!("RPC Error: {}", serde_json::to_string_pretty(error)?);
    }

    Ok(())
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
}
