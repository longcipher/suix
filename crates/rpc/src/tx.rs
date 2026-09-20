use eyre::Result;
use serde_json::{Value, json};

use crate::{RpcConfig, print_response, rpc_call};

pub const DEFAULT_GAS_BUDGET: u64 = 50_000_000;

/// Options shared by unsigned transaction builders.
pub struct TxOptions<'a> {
    pub gas: Option<&'a str>,
    pub gas_budget: u64,
    pub dry_run: bool,
}

/// Build unsigned `unsafe_*` transaction bytes via the tx-builder API.
pub async fn build_unsigned(url: &str, method: &str, params: Value) -> Result<(String, u64, u64)> {
    let response = rpc_call(url, method, params).await?;
    if let Some(error) = response.get("error") {
        eyre::bail!("RPC error building transaction: {error}");
    }
    let result = response
        .get("result")
        .ok_or_else(|| eyre::eyre!("Missing result in RPC response"))?;
    let tx_bytes = result
        .get("txBytes")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("Missing txBytes in RPC response"))?;
    let gas_budget = result
        .get("gasBudget")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_GAS_BUDGET);
    let gas_price = result
        .get("gasPrice")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    Ok((tx_bytes.to_string(), gas_budget, gas_price))
}

/// Print unsigned transaction bytes, optionally executing after signing.
pub fn print_unsigned(tx_bytes: &str, gas_budget: u64, gas_price: u64, dry_run: bool) {
    println!("Unsigned tx bytes (base64): {tx_bytes}");
    println!("Gas budget: {gas_budget}");
    println!("Gas price: {gas_price}");
    if dry_run {
        println!(
            "Dry-run requested: submit with `suix tx submit --tx-bytes {tx_bytes} --signatures [...]` after signing, or pipe through `suix tx dry-run --tx-bytes {tx_bytes}`."
        );
    } else {
        println!(
            "Sign with `suix key sign-tx --tx-bytes {tx_bytes}`, then `suix tx submit --tx-bytes {tx_bytes} --signatures <SIG...>`."
        );
    }
}

fn opt_object_id(gas: Option<&str>) -> Value {
    gas.map_or(Value::Null, |g| Value::String(g.to_string()))
}

pub async fn transfer_object(
    config: &RpcConfig,
    signer: &str,
    object_id: &str,
    recipient: &str,
    options: TxOptions<'_>,
) -> Result<()> {
    let params = json!([
        signer,
        object_id,
        opt_object_id(options.gas),
        options.gas_budget.to_string(),
        recipient
    ]);
    let (tx_bytes, budget, price) =
        build_unsigned(&config.url, "unsafe_transferObject", params).await?;
    print_unsigned(&tx_bytes, budget, price, options.dry_run);
    Ok(())
}

pub async fn transfer_sui(
    config: &RpcConfig,
    signer: &str,
    sui_object_id: &str,
    recipient: &str,
    amount: Option<u64>,
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([
        signer,
        sui_object_id,
        gas_budget.to_string(),
        recipient,
        amount.map(|a| a.to_string())
    ]);
    let (tx_bytes, budget, price) =
        build_unsigned(&config.url, "unsafe_transferSui", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

pub async fn pay(
    config: &RpcConfig,
    signer: &str,
    input_coins: &[String],
    recipients: &[String],
    amounts: &[u64],
    options: TxOptions<'_>,
) -> Result<()> {
    let params = json!([
        signer,
        input_coins,
        recipients,
        amounts.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
        opt_object_id(options.gas),
        options.gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) = build_unsigned(&config.url, "unsafe_pay", params).await?;
    print_unsigned(&tx_bytes, budget, price, options.dry_run);
    Ok(())
}

pub async fn pay_sui(
    config: &RpcConfig,
    signer: &str,
    input_coins: &[String],
    recipients: &[String],
    amounts: &[u64],
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([
        signer,
        input_coins,
        recipients,
        amounts.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
        gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) = build_unsigned(&config.url, "unsafe_paySui", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

pub async fn pay_all_sui(
    config: &RpcConfig,
    signer: &str,
    input_coins: &[String],
    recipient: &str,
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([signer, input_coins, recipient, gas_budget.to_string()]);
    let (tx_bytes, budget, price) = build_unsigned(&config.url, "unsafe_payAllSui", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

/// Arguments for a generic Move call.
pub struct MoveCallArgs<'a> {
    pub package: &'a str,
    pub module: &'a str,
    pub function: &'a str,
    pub type_args: &'a [String],
    pub args_json: &'a str,
}

pub async fn move_call(
    config: &RpcConfig,
    signer: &str,
    call: MoveCallArgs<'_>,
    options: TxOptions<'_>,
) -> Result<()> {
    let args: Value = serde_json::from_str(call.args_json)
        .map_err(|e| eyre::eyre!("Invalid JSON args array: {}", e))?;
    let params = json!([
        signer,
        call.package,
        call.module,
        call.function,
        call.type_args,
        args,
        opt_object_id(options.gas),
        options.gas_budget.to_string(),
        null
    ]);
    let (tx_bytes, budget, price) = build_unsigned(&config.url, "unsafe_moveCall", params).await?;
    print_unsigned(&tx_bytes, budget, price, options.dry_run);
    Ok(())
}

pub async fn split_coin(
    config: &RpcConfig,
    signer: &str,
    coin_object_id: &str,
    amounts: &[u64],
    gas: Option<&str>,
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([
        signer,
        coin_object_id,
        amounts.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
        opt_object_id(gas),
        gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) = build_unsigned(&config.url, "unsafe_splitCoin", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

pub async fn split_coin_equal(
    config: &RpcConfig,
    signer: &str,
    coin_object_id: &str,
    split_count: u64,
    gas: Option<&str>,
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([
        signer,
        coin_object_id,
        split_count.to_string(),
        opt_object_id(gas),
        gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) =
        build_unsigned(&config.url, "unsafe_splitCoinEqual", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

pub async fn merge_coins(
    config: &RpcConfig,
    signer: &str,
    primary_coin: &str,
    coin_to_merge: &str,
    gas: Option<&str>,
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([
        signer,
        primary_coin,
        coin_to_merge,
        opt_object_id(gas),
        gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) =
        build_unsigned(&config.url, "unsafe_mergeCoins", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

pub async fn publish(
    config: &RpcConfig,
    sender: &str,
    modules_b64: &[String],
    dependencies: &[String],
    gas: Option<&str>,
    gas_budget: u64,
    dry_run: bool,
) -> Result<()> {
    let params = json!([
        sender,
        modules_b64,
        dependencies,
        opt_object_id(gas),
        gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) = build_unsigned(&config.url, "unsafe_publish", params).await?;
    print_unsigned(&tx_bytes, budget, price, dry_run);
    Ok(())
}

/// Arguments for delegating stake.
pub struct DelegateArgs<'a> {
    pub coins: &'a [String],
    pub amount: Option<u64>,
    pub validator: &'a str,
}

pub async fn request_add_stake(
    config: &RpcConfig,
    signer: &str,
    delegate: DelegateArgs<'_>,
    options: TxOptions<'_>,
) -> Result<()> {
    let params = json!([
        signer,
        delegate.coins,
        delegate.amount.map(|a| a.to_string()),
        delegate.validator,
        opt_object_id(options.gas),
        options.gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) =
        build_unsigned(&config.url, "unsafe_requestAddStake", params).await?;
    print_unsigned(&tx_bytes, budget, price, options.dry_run);
    Ok(())
}

pub async fn request_withdraw_stake(
    config: &RpcConfig,
    signer: &str,
    staked_sui: &str,
    options: TxOptions<'_>,
) -> Result<()> {
    let params = json!([
        signer,
        staked_sui,
        opt_object_id(options.gas),
        options.gas_budget.to_string()
    ]);
    let (tx_bytes, budget, price) =
        build_unsigned(&config.url, "unsafe_requestWithdrawStake", params).await?;
    print_unsigned(&tx_bytes, budget, price, options.dry_run);
    Ok(())
}

/// Wait for a transaction to appear in a checkpoint (local execution).
pub async fn wait_for_tx(config: &RpcConfig, digest: &str, timeout_secs: u64) -> Result<()> {
    use tokio::time::{Duration, sleep};

    let params = |d: &str| {
        format!(
            r#"["{d}", {{"showInput": false, "showRawInput": false, "showEffects": true, "showEvents": false, "showObjectChanges": false, "showBalanceChanges": false}}]"#
        )
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        let response = rpc_call(
            &config.url,
            "sui_getTransactionBlock",
            serde_json::from_str(&format!(
                "[{}]",
                params(digest).trim_start_matches('[').trim_end_matches(']')
            ))?,
        )
        .await?;
        if response.get("result").is_some() {
            println!("Transaction {digest} confirmed on {}", config.url);
            print_response(&response, config.pretty)?;
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            eyre::bail!("Timed out waiting for transaction {digest}");
        }
        sleep(Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_object_id() {
        assert_eq!(opt_object_id(None), Value::Null);
        assert_eq!(opt_object_id(Some("0x1")), json!("0x1"));
    }
}
