use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;
use sui_rpc_api::Client;
use sui_rpc_api::client::HeadersInterceptor;
use sui_rpc_api::proto::sui::rpc::v2 as proto;

#[derive(Debug, Clone)]
pub struct GrpcConfig {
    pub url: String,
    pub pretty: bool,
    pub json: bool,
    pub timeout: Duration,
    pub headers: Vec<(String, String)>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            url: "https://fullnode.mainnet.sui.io:443".to_string(),
            pretty: false,
            json: false,
            timeout: Duration::from_secs(30),
            headers: vec![],
        }
    }
}

/// Raw gRPC service and method call structure
#[derive(Debug, Clone)]
pub struct GrpcCall {
    pub service: String,
    pub method: String,
    pub data: Option<Value>,
}

pub struct SuiGrpcClient {
    config: GrpcConfig,
    client: Client,
}

impl SuiGrpcClient {
    pub async fn new(config: GrpcConfig) -> Result<Self> {
        if !config.json {
            println!("Creating Sui gRPC client for: {}", config.url);
        }

        let mut client = Client::new(&config.url)
            .map_err(|e| anyhow::anyhow!("Failed to create gRPC client: {}", e))?;

        if !config.headers.is_empty() {
            let mut interceptor = HeadersInterceptor::new();
            {
                let metadata = interceptor.headers_mut();
                for (k, v) in &config.headers {
                    let key: tonic::metadata::MetadataKey<_> = k
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Invalid header name {k}: {e}"))?;
                    let value: tonic::metadata::MetadataValue<_> = v
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Invalid header value for {k}: {e}"))?;
                    metadata.insert(key, value);
                }
            }
            client = client.with_headers(interceptor);
        }

        if !config.json {
            println!("Sui gRPC client created successfully");
        }
        Ok(Self { config, client })
    }

    pub fn config(&self) -> &GrpcConfig {
        &self.config
    }

    fn emit_checkpoint_summary_json(
        &self,
        seq: u64,
        epoch: u64,
        digest: &str,
        network_total_transactions: u64,
        timestamp_ms: u64,
        event_type: &str,
    ) -> Result<()> {
        let json_output = serde_json::json!({
            "sequence_number": seq,
            "epoch": epoch,
            "digest": digest,
            "network_total_transactions": network_total_transactions,
            "timestamp_ms": timestamp_ms,
            "event_type": event_type
        });
        println!("{}", serde_json::to_string(&json_output)?);
        Ok(())
    }

    pub async fn get_service_info(&mut self) -> Result<()> {
        if !self.config.json {
            println!("Fetching service info using sui-rpc-api gRPC client...");
        }

        match self.client.get_latest_checkpoint().await {
            Ok(checkpoint) => {
                if self.config.json {
                    self.emit_checkpoint_summary_json(
                        *checkpoint.sequence_number(),
                        checkpoint.epoch(),
                        &checkpoint.digest().to_string(),
                        checkpoint.network_total_transactions,
                        checkpoint.timestamp_ms,
                        "checkpoint",
                    )?;
                } else if self.config.pretty {
                    println!("Latest Checkpoint: {checkpoint:#?}");
                } else {
                    println!("Latest Checkpoint: {checkpoint:?}");
                }

                if !self.config.json {
                    println!("Sui gRPC service info retrieved successfully!");
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get latest checkpoint: {}", e));
            }
        }

        Ok(())
    }

    /// Get latest checkpoint using actual gRPC call
    pub async fn get_latest_checkpoint(&mut self) -> Result<()> {
        match self.client.get_latest_checkpoint().await {
            Ok(checkpoint) => {
                if self.config.json {
                    self.emit_checkpoint_summary_json(
                        *checkpoint.sequence_number(),
                        checkpoint.epoch(),
                        &checkpoint.digest().to_string(),
                        checkpoint.network_total_transactions,
                        checkpoint.timestamp_ms,
                        "checkpoint",
                    )?;
                } else if self.config.pretty {
                    println!("Latest Checkpoint Summary:");
                    println!("  Sequence Number: {}", checkpoint.sequence_number());
                    println!("  Digest: {}", checkpoint.digest());
                    println!("  Epoch: {}", checkpoint.epoch());
                    println!("  Previous Digest: {:?}", checkpoint.previous_digest);
                    println!(
                        "  Network Total Transactions: {}",
                        checkpoint.network_total_transactions
                    );
                } else {
                    println!(
                        "Checkpoint: sequence={}, epoch={}, digest={}",
                        checkpoint.sequence_number(),
                        checkpoint.epoch(),
                        checkpoint.digest()
                    );
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get latest checkpoint: {}", e)),
        }
    }

    /// Get checkpoint by sequence number
    pub async fn get_checkpoint(&mut self, sequence_number: u64) -> Result<()> {
        match self.client.get_checkpoint_summary(sequence_number).await {
            Ok(checkpoint) => {
                if self.config.pretty {
                    println!("Checkpoint Summary: {checkpoint:#?}");
                } else {
                    println!("Checkpoint: {checkpoint:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!(
                "Failed to get checkpoint {}: {}",
                sequence_number,
                e
            )),
        }
    }

    /// Get object by ID
    pub async fn get_object(&mut self, object_id: &str) -> Result<()> {
        let object_id = object_id
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid object ID: {}", e))?;

        match self.client.get_object(object_id).await {
            Ok(object) => {
                if self.config.json {
                    let json_output = serde_json::json!({
                        "object_id": object_id.to_string(),
                        "version": object.version().value(),
                        "digest": object.digest().to_string(),
                        "type": object.type_().map(|t| t.to_string()),
                        "owner": format!("{:?}", object.owner()),
                        "previous_transaction": object.previous_transaction.to_string(),
                        "storage_rebate": object.storage_rebate,
                    });
                    println!("{}", serde_json::to_string(&json_output)?);
                } else if self.config.pretty {
                    println!("Object: {object:#?}");
                } else {
                    println!("Object: {object:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get object: {}", e)),
        }
    }

    /// Get object at a specific version
    pub async fn get_object_with_version(&mut self, object_id: &str, version: u64) -> Result<()> {
        let object_id = object_id
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid object ID: {}", e))?;

        match self
            .client
            .get_object_with_version(object_id, version.into())
            .await
        {
            Ok(object) => {
                if self.config.pretty {
                    println!("Object: {object:#?}");
                } else {
                    println!("Object: {object:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get object version: {}", e)),
        }
    }

    /// Get transaction by digest (real implementation)
    pub async fn get_transaction(&mut self, digest: &str) -> Result<()> {
        use sui_types::effects::TransactionEffectsAPI;

        let digest = digest
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid transaction digest: {}", e))?;

        match self.client.get_transaction(&digest).await {
            Ok(tx) => {
                if self.config.json {
                    let out = serde_json::json!({
                        "digest": digest.to_string(),
                        "checkpoint": tx.checkpoint,
                        "timestamp_ms": tx.timestamp_ms(),
                        "effects_status": format!("{:?}", tx.effects.status()),
                        "events_count": tx.events.as_ref().map(|e| e.data.len()),
                        "balance_changes": tx.balance_changes.len(),
                        "changed_objects": tx.changed_objects.len(),
                    });
                    println!("{}", serde_json::to_string(&out)?);
                } else if self.config.pretty {
                    println!("Transaction {digest}: {tx:#?}");
                } else {
                    println!("Transaction {digest}: {tx:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get transaction: {}", e)),
        }
    }

    /// Get SUI balance of an address for a coin type
    pub async fn get_balance(&mut self, address: &str, coin_type: Option<&str>) -> Result<()> {
        use std::str::FromStr;
        let owner: sui_types::base_types::SuiAddress = address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
        let coin_type_str = coin_type.unwrap_or(
            "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI",
        );
        let struct_tag = move_core_types::language_storage::StructTag::from_str(coin_type_str)
            .map_err(|e| anyhow::anyhow!("Invalid coin type: {}", e))?;

        match self.client.get_balance(owner, &struct_tag).await {
            Ok(balance) => {
                let coin_type = balance.coin_type_opt().unwrap_or(coin_type_str);
                let total = balance.balance_opt().unwrap_or(0);
                if self.config.json {
                    let out = serde_json::json!({
                        "address": address,
                        "coin_type": coin_type,
                        "balance": total,
                    });
                    println!("{}", serde_json::to_string(&out)?);
                } else if self.config.pretty {
                    println!("Balance for {address} ({coin_type}): {balance:#?}");
                } else {
                    println!("Balance for {address} ({coin_type}): {total}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get balance: {}", e)),
        }
    }

    /// List all balances of an address (streaming list)
    pub async fn list_balances(&mut self, address: &str) -> Result<()> {
        let owner: sui_types::base_types::SuiAddress = address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

        let mut stream = Box::pin(self.client.list_balances(owner));
        let mut count = 0u64;
        while let Some(item) = stream.next().await {
            match item {
                Ok(balance) => {
                    count += 1;
                    let coin_type = balance.coin_type_opt().unwrap_or("unknown");
                    let total = balance.balance_opt().unwrap_or(0);
                    if self.config.json {
                        println!(
                            "{}",
                            serde_json::to_string(&serde_json::json!({
                                "coin_type": coin_type,
                                "balance": total,
                            }))?
                        );
                    } else if self.config.pretty {
                        println!("Balance #{count}: {balance:#?}");
                    } else {
                        println!("Balance #{count}: {coin_type} = {total}");
                    }
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to list balances: {}", e)),
            }
        }
        if !self.config.json {
            println!("Total balances: {count}");
        }
        Ok(())
    }

    /// List objects owned by an address
    pub async fn list_owned_objects(
        &mut self,
        address: &str,
        object_type: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<()> {
        use std::str::FromStr;
        let owner: sui_types::base_types::SuiAddress = address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
        let object_type = object_type
            .map(move_core_types::language_storage::StructTag::from_str)
            .transpose()
            .map_err(|e| anyhow::anyhow!("Invalid object type: {}", e))?;

        match self
            .client
            .get_owned_objects(owner, object_type, page_size, None)
            .await
        {
            Ok(page) => {
                if self.config.json {
                    for object in &page.items {
                        println!(
                            "{}",
                            serde_json::to_string(&serde_json::json!({
                                "object_id": object.id().to_string(),
                                "version": object.version().value(),
                                "digest": object.digest().to_string(),
                                "type": object.type_().map(|t| t.to_string()),
                                "owner": format!("{:?}", object.owner()),
                            }))?
                        );
                    }
                } else if self.config.pretty {
                    for object in &page.items {
                        println!("Object: {object:#?}");
                    }
                } else {
                    for object in &page.items {
                        println!(
                            "Object {} version={} type={:?}",
                            object.id(),
                            object.version().value(),
                            object.type_().map(|t| t.to_string())
                        );
                    }
                }
                if !self.config.json {
                    println!("Total objects: {}", page.items.len());
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to list owned objects: {}", e)),
        }
    }

    /// Get dynamic fields of a parent object
    pub async fn get_dynamic_fields(&mut self, parent: &str, page_size: Option<u32>) -> Result<()> {
        let parent = parent
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid parent object ID: {}", e))?;

        match self
            .client
            .get_dynamic_fields(parent, page_size, None)
            .await
        {
            Ok(response) => {
                if self.config.pretty {
                    println!("Dynamic fields: {response:#?}");
                } else {
                    println!("Dynamic fields: {response:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get dynamic fields: {}", e)),
        }
    }

    /// Get chain identifier
    pub async fn get_chain_identifier(&mut self) -> Result<()> {
        match self.client.get_chain_identifier().await {
            Ok(chain_id) => {
                if self.config.json {
                    println!(
                        "{}",
                        serde_json::to_string(&serde_json::json!({
                            "chain_identifier": chain_id.to_string(),
                        }))?
                    );
                } else {
                    println!("Chain identifier: {chain_id}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get chain identifier: {}", e)),
        }
    }

    /// Get reference gas price
    pub async fn get_reference_gas_price(&mut self) -> Result<()> {
        match self.client.get_reference_gas_price().await {
            Ok(price) => {
                if self.config.json {
                    println!(
                        "{}",
                        serde_json::to_string(&serde_json::json!({
                            "reference_gas_price": price,
                        }))?
                    );
                } else {
                    println!("Reference gas price: {price}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get reference gas price: {}", e)),
        }
    }

    /// Get delegated stakes for an owner
    pub async fn list_delegated_stake(&mut self, address: &str) -> Result<()> {
        let owner: sui_types::base_types::SuiAddress = address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

        match self.client.list_delegated_stake(owner).await {
            Ok(stakes) => {
                if self.config.json {
                    let items: Vec<serde_json::Value> = stakes
                        .iter()
                        .map(|s| {
                            serde_json::json!({
                                "staked_sui_id": s.staked_sui_id.to_string(),
                                "validator": s.validator_address.to_string(),
                                "pool": s.staking_pool.to_string(),
                                "activation_epoch": s.activation_epoch,
                                "principal": s.principal,
                                "rewards": s.rewards,
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string(&serde_json::json!(items))?);
                } else if self.config.pretty {
                    println!("Delegated stakes for {address}: {stakes:#?}");
                } else {
                    for stake in &stakes {
                        println!(
                            "Stake {} validator={} principal={} rewards={}",
                            stake.staked_sui_id,
                            stake.validator_address,
                            stake.principal,
                            stake.rewards
                        );
                    }
                    println!("Total stakes: {}", stakes.len());
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to list delegated stake: {}", e)),
        }
    }

    /// Get system state summary
    pub async fn get_system_state(&mut self, epoch: Option<u64>) -> Result<()> {
        match self.client.get_system_state(epoch).await {
            Ok(state) => {
                if self.config.pretty {
                    println!("System state: {state:#?}");
                } else {
                    println!("System state: {state:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to get system state: {}", e)),
        }
    }

    /// Simulate a transaction from base64 tx bytes
    pub async fn simulate_transaction(&mut self, tx_bytes_b64: &str) -> Result<()> {
        use fastcrypto::encoding::{Base64, Encoding};
        use sui_types::transaction::TransactionData;

        let bytes = Base64::decode(tx_bytes_b64)
            .map_err(|e| anyhow::anyhow!("Invalid base64 tx bytes: {}", e))?;
        let tx_data: TransactionData = bcs::from_bytes(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to decode TransactionData: {}", e))?;

        match self
            .client
            .simulate_transaction(&tx_data, true, false)
            .await
        {
            Ok(response) => {
                if self.config.pretty {
                    println!("Simulation result: {response:#?}");
                } else {
                    println!("Simulation result: {response:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to simulate transaction: {}", e)),
        }
    }

    /// Execute a signed transaction (base64 tx bytes + base64 signatures)
    pub async fn execute_transaction(
        &mut self,
        tx_bytes_b64: &str,
        signatures_b64: &[String],
        wait_for_checkpoint: bool,
    ) -> Result<()> {
        use fastcrypto::encoding::{Base64, Encoding};
        use sui_types::effects::TransactionEffectsAPI;
        use sui_types::transaction::{Transaction, TransactionData};

        let tx_bytes = Base64::decode(tx_bytes_b64)
            .map_err(|e| anyhow::anyhow!("Invalid base64 tx bytes: {}", e))?;
        let tx_data: TransactionData = bcs::from_bytes(&tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to decode TransactionData: {}", e))?;

        let mut signatures = Vec::with_capacity(signatures_b64.len());
        for sig in signatures_b64 {
            let bytes = Base64::decode(sig)
                .map_err(|e| anyhow::anyhow!("Invalid base64 signature: {}", e))?;
            let generic: sui_types::signature::GenericSignature = bcs::from_bytes(&bytes)
                .map_err(|e| anyhow::anyhow!("Failed to decode signature: {}", e))?;
            signatures.push(generic);
        }

        let tx = Transaction::from_generic_sig_data(tx_data, signatures);
        let result = if wait_for_checkpoint {
            self.client
                .execute_transaction_and_wait_for_checkpoint(&tx)
                .await
        } else {
            self.client.execute_transaction(&tx).await
        };

        match result {
            Ok(executed) => {
                if self.config.json {
                    let out = serde_json::json!({
                        "checkpoint": executed.checkpoint,
                        "timestamp_ms": executed.timestamp_ms(),
                        "effects_status": format!("{:?}", executed.effects.status()),
                        "events_count": executed.events.as_ref().map(|e| e.data.len()),
                        "balance_changes": executed.balance_changes.len(),
                    });
                    println!("{}", serde_json::to_string(&out)?);
                } else if self.config.pretty {
                    println!("Executed transaction: {executed:#?}");
                } else {
                    println!("Executed transaction: {executed:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to execute transaction: {}", e)),
        }
    }

    /// Generic gRPC call - similar to buf curl functionality
    pub async fn call_grpc_method(&mut self, call: GrpcCall) -> Result<()> {
        println!("Calling gRPC method: {}.{}", call.service, call.method);

        match (call.service.as_str(), call.method.as_str()) {
            ("sui.rpc.v2beta2.LedgerService", "GetLatestCheckpoint")
            | ("sui.rpc.v2.LedgerService", "GetLatestCheckpoint") => self.get_service_info().await,
            ("sui.rpc.v2beta2.LedgerService", "GetCheckpoint")
            | ("sui.rpc.v2.LedgerService", "GetCheckpoint") => {
                if let Some(data) = call.data
                    && let Some(seq) = data.get("sequence_number")
                    && let Some(seq_num) = seq.as_u64()
                {
                    return self.get_checkpoint(seq_num).await;
                }
                Err(anyhow::anyhow!(
                    "GetCheckpoint requires sequence_number parameter"
                ))
            }
            ("sui.rpc.v2beta2.LedgerService", "GetObject")
            | ("sui.rpc.v2.LedgerService", "GetObject") => {
                if let Some(data) = call.data
                    && let Some(object_id) = data.get("object_id")
                    && let Some(id_str) = object_id.as_str()
                {
                    return self.get_object(id_str).await;
                }
                Err(anyhow::anyhow!("GetObject requires object_id parameter"))
            }
            ("sui.rpc.v2beta2.LedgerService", "GetFullCheckpoint")
            | ("sui.rpc.v2.LedgerService", "GetFullCheckpoint") => {
                if let Some(data) = call.data
                    && let Some(seq) = data.get("sequence_number")
                    && let Some(seq_num) = seq.as_u64()
                {
                    return self.get_full_checkpoint(seq_num).await;
                }
                Err(anyhow::anyhow!(
                    "GetFullCheckpoint requires sequence_number parameter"
                ))
            }
            ("sui.rpc.v2beta2.LedgerService", "SubscribeCheckpoints")
            | ("sui.rpc.v2.LedgerService", "SubscribeCheckpoints") => {
                self.subscribe_checkpoints().await
            }
            ("sui.rpc.v2beta2.LedgerService", "GetTransaction")
            | ("sui.rpc.v2.LedgerService", "GetTransaction") => {
                if let Some(data) = call.data
                    && let Some(digest) = data.get("digest")
                    && let Some(digest_str) = digest.as_str()
                {
                    return self.get_transaction(digest_str).await;
                }
                Err(anyhow::anyhow!("GetTransaction requires digest parameter"))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported gRPC method: {}.{}",
                call.service,
                call.method
            )),
        }
    }

    /// Get full checkpoint data (similar to buf curl example)
    pub async fn get_full_checkpoint(&mut self, sequence_number: u64) -> Result<()> {
        match self.client.get_full_checkpoint(sequence_number).await {
            Ok(checkpoint_data) => {
                if self.config.pretty {
                    println!("Full Checkpoint Data: {checkpoint_data:#?}");
                } else {
                    println!("Full Checkpoint: {checkpoint_data:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!(
                "Failed to get full checkpoint {}: {}",
                sequence_number,
                e
            )),
        }
    }

    /// Get full checkpoint and dump raw summary to a directory for offline analysis
    pub async fn dump_full_checkpoint(
        &mut self,
        sequence_number: u64,
        dir: &std::path::Path,
    ) -> Result<()> {
        let checkpoint = self
            .client
            .get_full_checkpoint(sequence_number)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get full checkpoint {sequence_number}: {e}"))?;
        std::fs::create_dir_all(dir)
            .map_err(|e| anyhow::anyhow!("Failed to create dump dir: {}", e))?;
        let path = dir.join(format!("checkpoint-{sequence_number}.debug.txt"));
        std::fs::write(&path, format!("{checkpoint:#?}"))
            .map_err(|e| anyhow::anyhow!("Failed to write dump file: {}", e))?;
        println!("Checkpoint {sequence_number} dumped to {}", path.display());
        Ok(())
    }

    /// Subscribe to checkpoint stream: try the real server-streaming RPC first,
    /// fall back to polling simulation when the server rejects streaming.
    pub async fn subscribe_checkpoints(&mut self) -> Result<()> {
        if !self.config.json {
            println!("Subscribing to checkpoint stream...");
        }

        match self.try_stream_checkpoints(None).await {
            Ok(_) => Ok(()),
            Err(_) => {
                if !self.config.json {
                    println!("Streaming not available, using polling simulation...");
                }
                self.simulate_checkpoint_subscription().await
            }
        }
    }

    /// Real server-streaming subscription against SubscriptionService.
    /// Takes the first `limit` checkpoints (None = unbounded) then returns.
    pub async fn try_stream_checkpoints(&mut self, limit: Option<u64>) -> Result<()> {
        let request = proto::SubscribeCheckpointsRequest::default();
        let mut stream = self
            .client
            .inner_mut()
            .subscription_client()
            .subscribe_checkpoints(request)
            .await
            .map_err(|e| anyhow::anyhow!("SubscribeCheckpoints failed: {}", e))?
            .into_inner();

        let mut count = 0u64;
        while let Some(item) = stream.next().await {
            let response = item.map_err(|e| anyhow::anyhow!("Stream error: {}", e))?;
            count += 1;
            if self.config.json {
                let out = serde_json::json!({
                    "cursor": response.cursor().to_string(),
                    "event_type": "checkpoint",
                });
                println!("{}", serde_json::to_string(&out)?);
            } else if self.config.pretty {
                println!("Checkpoint event #{count}: {response:#?}");
            } else {
                println!("Checkpoint event #{count}: cursor={}", response.cursor());
            }
            if let Some(limit) = limit
                && count >= limit
            {
                break;
            }
        }
        Ok(())
    }

    /// Simulate checkpoint subscription by polling
    async fn simulate_checkpoint_subscription(&mut self) -> Result<()> {
        let latest = self
            .client
            .get_latest_checkpoint()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get latest checkpoint: {}", e))?;

        let current_seq = latest.sequence_number();
        if !self.config.json {
            println!("Starting from checkpoint: {}", *current_seq);
        }

        for i in 0..5u64 {
            if *current_seq >= i {
                let seq = *current_seq - i;
                match self.client.get_checkpoint_summary(seq).await {
                    Ok(checkpoint) => {
                        if self.config.json {
                            self.emit_checkpoint_summary_json(
                                seq,
                                checkpoint.epoch(),
                                &checkpoint.digest().to_string(),
                                checkpoint.network_total_transactions,
                                checkpoint.timestamp_ms,
                                "checkpoint",
                            )?;
                        } else if self.config.pretty {
                            println!("Checkpoint {seq}: {checkpoint:#?}");
                        } else {
                            println!(
                                "Checkpoint {seq}: epoch={}, txs={}",
                                checkpoint.epoch(),
                                checkpoint.network_total_transactions
                            );
                        }
                    }
                    Err(e) => {
                        if !self.config.json {
                            eprintln!("Failed to get checkpoint {seq}: {e}");
                        }
                    }
                }
            }
        }

        if !self.config.json {
            println!("Checkpoint subscription simulation completed");
        }
        Ok(())
    }

    /// Subscribe to checkpoints continuously (streaming mode)
    pub async fn subscribe_checkpoints_continuous(&mut self, interval_secs: u64) -> Result<()> {
        use tokio::time::{Duration, sleep};

        // Prefer the real stream; only poll when the server cannot stream.
        if self.try_stream_checkpoints(None).await.is_ok() {
            return Ok(());
        }

        if !self.config.json {
            println!(
                "Streaming not available, polling every {interval_secs} seconds (Ctrl+C to stop)..."
            );
        }

        let mut last_seen_sequence;

        match self.client.get_latest_checkpoint().await {
            Ok(checkpoint) => {
                last_seen_sequence = *checkpoint.sequence_number();
                if !self.config.json {
                    println!("Starting from checkpoint: {last_seen_sequence}");
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get initial checkpoint: {}", e));
            }
        }

        loop {
            sleep(Duration::from_secs(interval_secs)).await;

            match self.client.get_latest_checkpoint().await {
                Ok(checkpoint) => {
                    let current_sequence = *checkpoint.sequence_number();

                    if current_sequence > last_seen_sequence {
                        for seq in (last_seen_sequence + 1)..=current_sequence {
                            match self.client.get_checkpoint_summary(seq).await {
                                Ok(cp) => {
                                    if self.config.json {
                                        self.emit_checkpoint_summary_json(
                                            seq,
                                            cp.epoch(),
                                            &cp.digest().to_string(),
                                            cp.network_total_transactions,
                                            cp.timestamp_ms,
                                            "new_checkpoint",
                                        )?;
                                    } else if self.config.pretty {
                                        println!("New Checkpoint {seq}: {cp:#?}");
                                    } else {
                                        println!(
                                            "New Checkpoint {seq}: epoch={}, txs={}, digest={}",
                                            cp.epoch(),
                                            cp.network_total_transactions,
                                            cp.digest()
                                        );
                                    }
                                }
                                Err(e) => {
                                    if !self.config.json {
                                        eprintln!("Failed to get checkpoint {seq}: {e}");
                                    }
                                }
                            }
                        }
                        last_seen_sequence = current_sequence;
                    } else if !self.config.json {
                        println!("No new checkpoints (current: {current_sequence})");
                    }
                }
                Err(e) => {
                    if !self.config.json {
                        eprintln!("Failed to get latest checkpoint: {e}");
                    }
                }
            }
        }
    }

    /// List available gRPC methods (similar to buf curl --list-methods)
    pub fn list_methods(&self) -> Vec<String> {
        vec![
            "sui.rpc.v2.LedgerService.GetLatestCheckpoint".to_string(),
            "sui.rpc.v2.LedgerService.GetCheckpoint".to_string(),
            "sui.rpc.v2.LedgerService.GetFullCheckpoint".to_string(),
            "sui.rpc.v2.LedgerService.GetObject".to_string(),
            "sui.rpc.v2.LedgerService.GetTransaction".to_string(),
            "sui.rpc.v2.SubscriptionService.SubscribeCheckpoints".to_string(),
            "sui.rpc.v2.SubscriptionService.SubscribeTransactions".to_string(),
            "sui.rpc.v2.StateService.GetBalance".to_string(),
            "sui.rpc.v2.StateService.ListBalances".to_string(),
            "sui.rpc.v2.StateService.ListOwnedObjects".to_string(),
            "sui.rpc.v2.StateService.ListDynamicFields".to_string(),
            "sui.rpc.v2.TransactionExecutionService.ExecuteTransaction".to_string(),
            "sui.rpc.v2.TransactionExecutionService.SimulateTransaction".to_string(),
        ]
    }

    /// Display available methods
    pub fn show_methods(&self) {
        println!("Available gRPC methods:");
        for method in self.list_methods() {
            println!("  {method}");
        }
    }

    /// Raw curl-like interface
    pub async fn curl(&mut self, service: &str, method: &str, data: Option<&str>) -> Result<()> {
        let parsed_data = if let Some(data_str) = data {
            Some(
                serde_json::from_str(data_str)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON data: {}", e))?,
            )
        } else {
            None
        };

        let call = GrpcCall {
            service: service.to_string(),
            method: method.to_string(),
            data: parsed_data,
        };

        self.call_grpc_method(call).await
    }
}

/// Additional helper methods
impl SuiGrpcClient {
    /// Test network connectivity
    pub async fn test_connection(&mut self) -> Result<bool> {
        match self.client.get_latest_checkpoint().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_config_default() {
        let config = GrpcConfig::default();
        assert_eq!(config.url, "https://fullnode.mainnet.sui.io:443");
        assert!(!config.pretty);
        assert_eq!(config.timeout, std::time::Duration::from_secs(30));
    }

    #[test]
    fn test_list_methods_covers_state_and_execution() {
        let _config = GrpcConfig::default();
        let methods = vec![
            "sui.rpc.v2.LedgerService.GetLatestCheckpoint".to_string(),
            "sui.rpc.v2.LedgerService.GetCheckpoint".to_string(),
            "sui.rpc.v2.LedgerService.GetFullCheckpoint".to_string(),
            "sui.rpc.v2.LedgerService.GetObject".to_string(),
            "sui.rpc.v2.LedgerService.GetTransaction".to_string(),
            "sui.rpc.v2.SubscriptionService.SubscribeCheckpoints".to_string(),
            "sui.rpc.v2.SubscriptionService.SubscribeTransactions".to_string(),
            "sui.rpc.v2.StateService.GetBalance".to_string(),
            "sui.rpc.v2.StateService.ListBalances".to_string(),
            "sui.rpc.v2.StateService.ListOwnedObjects".to_string(),
            "sui.rpc.v2.StateService.ListDynamicFields".to_string(),
            "sui.rpc.v2.TransactionExecutionService.ExecuteTransaction".to_string(),
            "sui.rpc.v2.TransactionExecutionService.SimulateTransaction".to_string(),
        ];
        assert_eq!(methods.len(), 13);
        assert!(methods.iter().all(|m| m.starts_with("sui.rpc.v2.")));
    }
}
