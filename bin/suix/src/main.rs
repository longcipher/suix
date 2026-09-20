use clap::Parser;
use eyre::Result;

mod commands;

use crate::commands::{
    Cli, Commands, check_pretty_json, faucet_url_for_profile, grpc_config_from, parse_headers,
    resolve_key_source, rpc_config_from_url, shorten_address,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let profile = cli.profile.as_deref();

    match cli.command {
        Commands::Vanity(args) => handlers::handle_vanity(args),
        Commands::JsonRpc(args) => handlers::handle_json_rpc(args, profile).await,
        Commands::Grpc(args) => handlers::handle_grpc(args, profile).await,
        Commands::JsonRpcQuick(cmd) => handlers::handle_query(cmd, profile).await,
        Commands::GrpcQuick(cmd) => handlers::handle_grpc_quick(cmd, profile).await,
        Commands::Key(cmd) => handlers::handle_key(cmd).await,
        Commands::Query(cmd) => handlers::handle_read(cmd, profile).await,
        Commands::Coin(cmd) => handlers::handle_coin(cmd, profile).await,
        Commands::Tx(cmd) => handlers::handle_tx(cmd, profile).await,
        Commands::Stake(cmd) => handlers::handle_stake(cmd, profile).await,
        Commands::System(cmd) => handlers::handle_system(cmd, profile).await,
        Commands::Util(cmd) => handlers::handle_util(cmd, profile).await,
        Commands::Graphql(args) => handlers::handle_graphql(args).await,
        Commands::Completion(args) => handlers::handle_completion(args),
    }
}

mod handlers {
    use super::*;

    pub fn handle_vanity(args: crate::commands::VanityArgs) -> Result<()> {
        use vanity::VanityConfig;

        if args.starts_with.is_none() && args.ends_with.is_none() && args.contains.is_none() {
            eyre::bail!("At least one of --starts-with, --ends-with, or --contains is required");
        }
        if args.count == 0 {
            eyre::bail!("Count must be greater than 0");
        }
        if args.addresses_per_round == 0 {
            eyre::bail!("Addresses per round must be greater than 0");
        }
        if let Some(ref save_path) = args.save_path {
            std::fs::create_dir_all(save_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perm = std::fs::Permissions::from_mode(0o700);
                std::fs::set_permissions(save_path, perm)?;
            }
        }

        let scheme = keystore::parse_scheme(&args.scheme)?;

        if args.estimate_only {
            for (label, pattern) in [
                ("starts-with", &args.starts_with),
                ("ends-with", &args.ends_with),
                ("contains", &args.contains),
            ]
            .iter()
            {
                if let Some(p) = pattern {
                    match vanity::pattern_nibbles(p) {
                        Some(n) => println!(
                            "{label} '{p}': ~{:.0} trials expected (16^{n})",
                            vanity::estimate_difficulty(n)
                        ),
                        None => println!("{label} '{p}': regex pattern, difficulty varies"),
                    }
                }
            }
            return Ok(());
        }

        let config = VanityConfig {
            starts_with: args.starts_with,
            ends_with: args.ends_with,
            contains: args.contains,
            scheme,
            save_path: args.save_path.map(|p| p.to_string_lossy().to_string()),
            threads: args.threads,
            max_addresses: args.count,
            addresses_per_round: args.addresses_per_round,
        };
        vanity::generate_vanity_addresses(&config)
    }

    pub async fn handle_json_rpc(
        args: crate::commands::JsonRpcArgs,
        profile: Option<&str>,
    ) -> Result<()> {
        let config = rpc_config_from_url(&args.url, args.pretty, profile);
        rpc::make_rpc_call(&config, &args.method, args.params.as_deref()).await
    }

    pub async fn handle_grpc(args: crate::commands::GrpcArgs, profile: Option<&str>) -> Result<()> {
        check_pretty_json(args.pretty, args.json)?;
        let mut url = args.url.clone();
        if profile.is_some() {
            let mut rpc = String::new();
            crate::commands::apply_profile(profile, &mut rpc, Some(&mut url));
            if args.url == crate::commands::grpc_url_default() {
                // keep profile URL
            } else {
                url = args.url.clone();
            }
        }
        let config = grpc::GrpcConfig {
            url,
            pretty: args.pretty,
            json: args.json,
            timeout: std::time::Duration::from_secs(args.timeout),
            headers: parse_headers(&args.header)?,
        };
        let mut client = grpc::SuiGrpcClient::new(config)
            .await
            .map_err(|e| eyre::eyre!("{e}"))?;
        client
            .curl(&args.service, &args.method, args.params.as_deref())
            .await
            .map_err(|e| eyre::eyre!("{e}"))
    }

    pub async fn handle_query(
        cmd: crate::commands::QueryCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::QueryCommands as Q;
        use rpc::methods;

        match cmd {
            Q::Chain(a) => {
                methods::get_chain_identifier(&rpc_config_from_url(&a.url, a.pretty, profile)).await
            }
            Q::Checkpoint(a) => {
                methods::get_latest_checkpoint_sequence_number(&rpc_config_from_url(
                    &a.url, a.pretty, profile,
                ))
                .await
            }
            Q::Object(a) => {
                methods::get_object(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.object_id,
                )
                .await
            }
            Q::Tx(a) => {
                methods::get_transaction_block(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.digest,
                )
                .await
            }
            Q::Balance(a) => {
                methods::get_balance(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    a.coin_type.as_deref(),
                )
                .await
            }
            Q::Balances(a) => {
                methods::get_all_balances(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                )
                .await
            }
            Q::Coins(a) => {
                methods::get_coins(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    a.coin_type.as_deref(),
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            Q::AllCoins(a) => {
                methods::get_all_coins(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            Q::Owned(a) => {
                methods::get_owned_objects(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            Q::DynamicFields(a) => {
                methods::get_dynamic_fields(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.parent_id,
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            Q::Events(a) => {
                methods::query_events(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.filter.as_deref(),
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            Q::Stakes(a) => {
                methods::get_stakes(&rpc_config_from_url(&a.url, a.pretty, profile), &a.address)
                    .await
            }
            Q::PastObject(a) => {
                methods::try_get_past_object(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.object_id,
                    a.version,
                )
                .await
            }
            Q::CheckpointById(a) => {
                methods::get_checkpoint(&rpc_config_from_url(&a.url, a.pretty, profile), &a.id)
                    .await
            }
            Q::Checkpoints(a) => {
                methods::get_checkpoints(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.cursor.as_deref(),
                    Some(a.limit),
                )
                .await
            }
            Q::Txs(a) => {
                methods::query_transactions(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.filter.as_deref(),
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            Q::CoinMetadata(a) => {
                methods::get_coin_metadata(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.coin_type,
                )
                .await
            }
            Q::TotalSupply(a) => {
                methods::get_total_supply(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.coin_type,
                )
                .await
            }
            Q::Package(a) => {
                methods::get_normalized_move_modules(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.package_id,
                )
                .await
            }
            Q::FunctionArgs(a) => {
                methods::get_move_function_arg_types(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.package_id,
                    &a.module,
                    &a.function,
                )
                .await
            }
            Q::Protocol(a) => {
                methods::get_protocol_config(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.version,
                )
                .await
            }
            Q::DryRun(a) => {
                methods::dry_run(&rpc_config_from_url(&a.url, a.pretty, profile), &a.tx_bytes).await
            }
            Q::DevInspect(a) => {
                methods::dev_inspect(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.sender,
                    &a.package,
                    &a.module,
                    &a.function,
                    &a.type_args,
                    &a.args,
                )
                .await
            }
        }
    }

    pub async fn handle_grpc_quick(
        cmd: crate::commands::GrpcCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::GrpcCommands as G;

        match cmd {
            G::Info(e) => {
                check_pretty_json(e.pretty, e.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&e, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_service_info()
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Object(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                if let Some(version) = a.version {
                    client
                        .get_object_with_version(&a.object_id, version)
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))
                } else {
                    client
                        .get_object(&a.object_id)
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))
                }
            }
            G::Tx(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_transaction(&a.digest)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Balance(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_balance(&a.address, a.coin_type.as_deref())
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Balances(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .list_balances(&a.address)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Owned(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .list_owned_objects(&a.address, a.object_type.as_deref(), a.page_size)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::DynamicFields(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_dynamic_fields(&a.parent_id, a.page_size)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Chain(e) => {
                check_pretty_json(e.pretty, e.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&e, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_chain_identifier()
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::GasPrice(e) => {
                check_pretty_json(e.pretty, e.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&e, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_reference_gas_price()
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Stakes(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .list_delegated_stake(&a.address)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::SystemState(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .get_system_state(a.epoch)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Simulate(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .simulate_transaction(&a.tx_bytes)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Execute(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .execute_transaction(&a.tx_bytes, &a.signatures, a.wait)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::Curl(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client
                    .curl(&a.service, &a.method, a.data.as_deref())
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))
            }
            G::ListMethods(a) => {
                let config = grpc::GrpcConfig {
                    url: a.url,
                    pretty: false,
                    json: false,
                    timeout: std::time::Duration::from_secs(30),
                    headers: vec![],
                };
                let client = grpc::SuiGrpcClient::new(config)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                client.show_methods();
                Ok(())
            }
            G::Subscribe(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                if a.stream {
                    client
                        .subscribe_checkpoints_continuous(a.interval)
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))
                } else {
                    client
                        .try_stream_checkpoints(a.limit.or(Some(5)))
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))
                }
            }
            G::FullCheckpoint(a) => {
                check_pretty_json(a.endpoint.pretty, a.endpoint.json)?;
                let mut client = grpc::SuiGrpcClient::new(grpc_config_from(&a.endpoint, profile)?)
                    .await
                    .map_err(|e| eyre::eyre!("{e}"))?;
                if let Some(dir) = a.dump_dir {
                    client
                        .get_full_checkpoint(a.sequence_number)
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))?;
                    client
                        .dump_full_checkpoint(a.sequence_number, &dir)
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))
                } else {
                    client
                        .get_full_checkpoint(a.sequence_number)
                        .await
                        .map_err(|e| eyre::eyre!("{e}"))
                }
            }
        }
    }

    pub async fn handle_key(cmd: crate::commands::KeyCommands) -> Result<()> {
        use crate::commands::KeyCommands as K;

        match cmd {
            K::Generate(a) => {
                let scheme = keystore::parse_scheme(&a.scheme)?;
                keystore::generate(scheme, a.word_length)
            }
            K::Import(a) => {
                let secret = a.secret.map_or_else(
                    || {
                        std::env::var("SUIX_KEY")
                            .map_err(|_| eyre::eyre!("Missing --secret and SUIX_KEY is not set"))
                    },
                    Ok,
                )?;
                keystore::import_to_keystore(a.keystore, &secret, a.alias)
            }
            K::Export(a) => keystore::export_from_keystore(a.keystore, &a.address),
            K::List(a) => keystore::list_keystore(a.keystore),
            K::Sign(a) => {
                let keypair = resolve_key_source(&a.key)?;
                let bytes = hex::decode(a.message.trim_start_matches("0x"))
                    .map_err(|e| eyre::eyre!("Invalid hex message: {}", e))?;
                keystore::sign_bytes(&keypair, &bytes)
            }
            K::SignTx(a) => {
                let keypair = resolve_key_source(&a.key)?;
                keystore::sign_transaction_data(&keypair, &a.tx_bytes)
            }
            K::DecodeSig(a) => keystore::decode_signature(&a.signature),
            K::MultisigAddress(a) => {
                if a.pks.is_empty() {
                    eyre::bail!("At least one --pk is required");
                }
                keystore::multisig_address(&a.pks, &a.weights, a.threshold)
            }
        }
    }

    pub async fn handle_read(
        cmd: crate::commands::ReadCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::ReadCommands as R;
        use rpc::methods;

        match cmd {
            R::Object(a) => {
                methods::get_object(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.object_id,
                )
                .await
            }
            R::Owned(a) => {
                methods::get_owned_objects(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            R::PastObject(a) => {
                methods::try_get_past_object(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.object_id,
                    a.version,
                )
                .await
            }
            R::DynamicFields(a) => {
                methods::get_dynamic_fields(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.parent_id,
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            R::Tx(a) => {
                methods::get_transaction_block(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.digest,
                )
                .await
            }
            R::Txs(a) => {
                methods::query_transactions(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.filter.as_deref(),
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            R::Events(a) => {
                methods::query_events(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.filter.as_deref(),
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            R::Checkpoint(a) => {
                methods::get_checkpoint(&rpc_config_from_url(&a.url, a.pretty, profile), &a.id)
                    .await
            }
            R::Checkpoints(a) => {
                methods::get_checkpoints(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.cursor.as_deref(),
                    Some(a.limit),
                )
                .await
            }
            R::Package(a) => {
                methods::get_normalized_move_modules(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.package_id,
                )
                .await
            }
            R::FunctionArgs(a) => {
                methods::get_move_function_arg_types(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.package_id,
                    &a.module,
                    &a.function,
                )
                .await
            }
        }
    }

    pub async fn handle_coin(
        cmd: crate::commands::CoinCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::CoinCommands as C;
        use rpc::methods;

        match cmd {
            C::Balance(a) => {
                methods::get_balance(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    a.coin_type.as_deref(),
                )
                .await
            }
            C::Balances(a) => {
                methods::get_all_balances(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                )
                .await
            }
            C::Coins(a) => {
                methods::get_coins(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    a.coin_type.as_deref(),
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            C::AllCoins(a) => {
                methods::get_all_coins(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                    Some(a.limit),
                    a.cursor.as_deref(),
                )
                .await
            }
            C::Metadata(a) => {
                methods::get_coin_metadata(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.coin_type,
                )
                .await
            }
            C::Supply(a) => {
                methods::get_total_supply(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.coin_type,
                )
                .await
            }
        }
    }

    pub async fn handle_tx(cmd: crate::commands::TxCommands, profile: Option<&str>) -> Result<()> {
        use crate::commands::TxCommands as T;
        use rpc::tx::{self, TxOptions};

        match cmd {
            T::Transfer(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::transfer_object(
                    &config,
                    &a.base.signer,
                    &a.object_id,
                    &a.to,
                    TxOptions {
                        gas: a.base.gas.as_deref(),
                        gas_budget: a.base.gas_budget,
                        dry_run: a.base.dry_run,
                    },
                )
                .await
            }
            T::TransferSui(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::transfer_sui(
                    &config,
                    &a.base.signer,
                    &a.coin,
                    &a.to,
                    a.amount,
                    a.base.gas_budget,
                    a.base.dry_run,
                )
                .await
            }
            T::Pay(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::pay(
                    &config,
                    &a.base.signer,
                    &a.coins,
                    &a.recipients,
                    &a.amounts,
                    TxOptions {
                        gas: a.base.gas.as_deref(),
                        gas_budget: a.base.gas_budget,
                        dry_run: a.base.dry_run,
                    },
                )
                .await
            }
            T::PaySui(a) => {
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                tx::pay_sui(
                    &config,
                    &a.signer,
                    &a.coins,
                    &a.recipients,
                    &a.amounts,
                    a.gas_budget,
                    a.dry_run,
                )
                .await
            }
            T::PayAllSui(a) => {
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                tx::pay_all_sui(&config, &a.signer, &a.coins, &a.to, a.gas_budget, a.dry_run).await
            }
            T::Call(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::move_call(
                    &config,
                    &a.base.signer,
                    tx::MoveCallArgs {
                        package: &a.package,
                        module: &a.module,
                        function: &a.function,
                        type_args: &a.type_args,
                        args_json: &a.args,
                    },
                    TxOptions {
                        gas: a.base.gas.as_deref(),
                        gas_budget: a.base.gas_budget,
                        dry_run: a.base.dry_run,
                    },
                )
                .await
            }
            T::Split(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::split_coin(
                    &config,
                    &a.base.signer,
                    &a.coin,
                    &a.amounts,
                    a.base.gas.as_deref(),
                    a.base.gas_budget,
                    a.base.dry_run,
                )
                .await
            }
            T::SplitEqual(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::split_coin_equal(
                    &config,
                    &a.base.signer,
                    &a.coin,
                    a.count,
                    a.base.gas.as_deref(),
                    a.base.gas_budget,
                    a.base.dry_run,
                )
                .await
            }
            T::Merge(a) => {
                let config = rpc_config_from_url(&a.base.url, a.base.pretty, profile);
                tx::merge_coins(
                    &config,
                    &a.base.signer,
                    &a.primary,
                    &a.merge,
                    a.base.gas.as_deref(),
                    a.base.gas_budget,
                    a.base.dry_run,
                )
                .await
            }
            T::Publish(a) => {
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                tx::publish(
                    &config,
                    &a.sender,
                    &a.modules,
                    &a.dependencies,
                    a.gas.as_deref(),
                    a.gas_budget,
                    a.dry_run,
                )
                .await
            }
            T::Submit(a) => {
                use rpc::methods;
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                let signatures = serde_json::to_string(&a.signatures)?;
                methods::execute_transaction(&config, &a.tx_bytes, &signatures).await
            }
            T::DryRun(a) => {
                use rpc::methods;
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                methods::dry_run(&config, &a.tx_bytes).await
            }
            T::Wait(a) => {
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                tx::wait_for_tx(&config, &a.digest, a.timeout).await
            }
        }
    }

    pub async fn handle_stake(
        cmd: crate::commands::StakeCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::StakeCommands as S;
        use rpc::{
            methods,
            tx::{self, TxOptions},
        };

        match cmd {
            S::Delegate(a) => {
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                tx::request_add_stake(
                    &config,
                    &a.signer,
                    tx::DelegateArgs {
                        coins: &a.coins,
                        amount: a.amount,
                        validator: &a.validator,
                    },
                    TxOptions {
                        gas: a.gas.as_deref(),
                        gas_budget: a.gas_budget,
                        dry_run: a.dry_run,
                    },
                )
                .await
            }
            S::Undelegate(a) => {
                let config = rpc_config_from_url(&a.url, a.pretty, profile);
                tx::request_withdraw_stake(
                    &config,
                    &a.signer,
                    &a.staked_sui,
                    TxOptions {
                        gas: a.gas.as_deref(),
                        gas_budget: a.gas_budget,
                        dry_run: a.dry_run,
                    },
                )
                .await
            }
            S::Rewards(a) => {
                methods::get_stakes(&rpc_config_from_url(&a.url, a.pretty, profile), &a.address)
                    .await
            }
        }
    }

    pub async fn handle_system(
        cmd: crate::commands::SystemCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::SystemCommands as S;
        use rpc::methods;

        match cmd {
            S::Chain(a) => {
                methods::get_chain_identifier(&rpc_config_from_url(&a.url, a.pretty, profile)).await
            }
            S::GasPrice(a) => {
                methods::get_reference_gas_price(&rpc_config_from_url(&a.url, a.pretty, profile))
                    .await
            }
            S::Committee(a) => {
                methods::get_committee_info(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.epoch,
                )
                .await
            }
            S::State(a) => {
                methods::get_latest_system_state(&rpc_config_from_url(&a.url, a.pretty, profile))
                    .await
            }
            S::Apy(a) => {
                methods::get_validators_apy(&rpc_config_from_url(&a.url, a.pretty, profile)).await
            }
            S::Epoch(a) => {
                methods::get_current_epoch(&rpc_config_from_url(&a.url, a.pretty, profile)).await
            }
            S::Protocol(a) => {
                methods::get_protocol_config(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    a.version,
                )
                .await
            }
            S::TotalTxs(a) => {
                methods::get_total_transactions(&rpc_config_from_url(&a.url, a.pretty, profile))
                    .await
            }
        }
    }

    pub async fn handle_util(
        cmd: crate::commands::UtilCommands,
        profile: Option<&str>,
    ) -> Result<()> {
        use crate::commands::UtilCommands as U;

        match cmd {
            U::DecodeTx(a) => {
                use fastcrypto::encoding::{Base64, Encoding};
                let bytes = Base64::decode(&a.tx_bytes)
                    .map_err(|e| eyre::eyre!("Invalid base64 tx bytes: {}", e))?;
                let tx_data: sui_types::transaction::TransactionData = bcs::from_bytes(&bytes)
                    .map_err(|e| eyre::eyre!("Invalid TransactionData: {}", e))?;
                println!("Digest: {}", tx_data.digest());
                println!("{tx_data:#?}");
                Ok(())
            }
            U::Address(a) => {
                if a.long {
                    println!("{}", a.address);
                } else {
                    println!("{}", shorten_address(&a.address));
                }
                Ok(())
            }
            U::Health(a) => {
                let rpc_config = rpc_config_from_url(&a.url, false, profile);
                let started = std::time::Instant::now();
                let chain = rpc::rpc_call(
                    &rpc_config.url,
                    "sui_getChainIdentifier",
                    serde_json::json!([]),
                )
                .await;
                match chain {
                    Ok(v) if v.get("result").is_some() => {
                        println!("JSON-RPC OK ({}ms): {v}", started.elapsed().as_millis())
                    }
                    Ok(v) => println!(
                        "JSON-RPC DEGRADED ({}ms): {v}",
                        started.elapsed().as_millis()
                    ),
                    Err(e) => println!("JSON-RPC FAILED: {e}"),
                }
                let grpc_url = a.grpc_url.unwrap_or_else(|| rpc_config.url.clone());
                let started = std::time::Instant::now();
                let config = grpc::GrpcConfig {
                    url: grpc_url.clone(),
                    pretty: false,
                    json: true,
                    timeout: std::time::Duration::from_secs(a.timeout),
                    headers: vec![],
                };
                match grpc::SuiGrpcClient::new(config).await {
                    Ok(mut client) => match client.test_connection().await {
                        Ok(true) => {
                            println!("gRPC OK ({}ms): {grpc_url}", started.elapsed().as_millis())
                        }
                        _ => println!("gRPC FAILED: {grpc_url}"),
                    },
                    Err(e) => println!("gRPC FAILED: {e}"),
                }
                Ok(())
            }
            U::Faucet(a) => {
                let url = a
                    .url
                    .unwrap_or_else(|| faucet_url_for_profile(profile).to_string());
                let client = reqwest::Client::new();
                let response = client
                    .post(&url)
                    .json(&serde_json::json!({"FixedAmountRequest": {"recipient": a.address}}))
                    .send()
                    .await
                    .map_err(|e| eyre::eyre!("Faucet request failed: {}", e))?;
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                if !status.is_success() {
                    eyre::bail!("Faucet failed ({status}): {text}");
                }
                println!("Faucet response: {text}");
                Ok(())
            }
            U::Resolve(a) => {
                rpc::methods::resolve_name(&rpc_config_from_url(&a.url, a.pretty, profile), &a.name)
                    .await
            }
            U::Reverse(a) => {
                rpc::methods::reverse_resolve(
                    &rpc_config_from_url(&a.url, a.pretty, profile),
                    &a.address,
                )
                .await
            }
        }
    }

    pub async fn handle_graphql(args: crate::commands::GraphqlArgs) -> Result<()> {
        let variables: serde_json::Value = serde_json::from_str(&args.variables)
            .map_err(|e| eyre::eyre!("Invalid JSON variables: {}", e))?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(args.timeout))
            .build()
            .map_err(|e| eyre::eyre!("{e}"))?;
        let response = client
            .post(&args.url)
            .json(&serde_json::json!({"query": args.query, "variables": variables}))
            .send()
            .await
            .map_err(|e| eyre::eyre!("GraphQL request failed: {}", e))?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            eyre::bail!("GraphQL failed ({status}): {text}");
        }
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| eyre::eyre!("Invalid JSON response: {}", e))?;
        if args.pretty {
            println!("{}", serde_json::to_string_pretty(&value)?);
        } else {
            println!("{value}");
        }
        if let Some(errors) = value.get("errors") {
            eprintln!("GraphQL errors: {errors}");
        }
        Ok(())
    }

    pub fn handle_completion(args: crate::commands::CompletionArgs) -> Result<()> {
        use clap::CommandFactory;
        use clap_complete::{Shell, generate};

        let shell: Shell = args.shell.parse().map_err(|e| eyre::eyre!("{e}"))?;
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "suix", &mut std::io::stdout());
        Ok(())
    }
}
