use std::path::PathBuf;

use eyre::{Context, Result};
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::traits::{EncodeDecodeBase64, Signer, ToFromBytes};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_keys::key_derive::generate_new_key;
use sui_keys::keypair_file::write_keypair_to_file;
use sui_types::base_types::SuiAddress;
use sui_types::crypto::{PublicKey, Signature, SignatureScheme, SuiKeyPair, SuiSignature};
use sui_types::multisig::{MultiSigPublicKey, WeightUnit};

/// Re-export for CLI key resolution.
pub use sui_types::crypto::SuiKeyPair as SuiKeyPairReexport;

/// Default keystore file managed by suix.
pub fn default_keystore_path() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| eyre::eyre!("Cannot locate home directory"))?;
    Ok(dir.join("suix").join("sui.keystore"))
}

/// Parse a signature scheme name.
pub fn parse_scheme(scheme: &str) -> Result<SignatureScheme> {
    match scheme.to_lowercase().as_str() {
        "ed25519" | "ed" => Ok(SignatureScheme::ED25519),
        "secp256k1" | "k1" => Ok(SignatureScheme::Secp256k1),
        "secp256r1" | "r1" => Ok(SignatureScheme::Secp256r1),
        _ => Err(eyre::eyre!(
            "Unknown signature scheme '{scheme}'; expected ed25519, secp256k1, or secp256r1"
        )),
    }
}

/// Load a keypair from a bech32 `suiprivkey...` string or base64 `flag || privkey`.
pub fn load_keypair(secret: &str) -> Result<SuiKeyPair> {
    let trimmed = secret.trim();
    if let Ok(kp) = SuiKeyPair::decode(trimmed) {
        return Ok(kp);
    }
    let bytes = Base64::decode(trimmed)
        .map_err(|e| eyre::eyre!("Invalid key format (expected suiprivkey or base64): {}", e))?;
    SuiKeyPair::from_bytes(&bytes).map_err(|e| eyre::eyre!("Invalid key bytes: {}", e))
}

/// Load a keypair from a file (bech32/base64 text or raw keypair file bytes).
pub fn load_keypair_from_file(path: &PathBuf) -> Result<SuiKeyPair> {
    let raw = std::fs::read(path)
        .wrap_err_with(|| format!("Failed to read key file {}", path.display()))?;
    if let Ok(text) = std::str::from_utf8(&raw) {
        let trimmed = text.trim();
        if !trimmed.is_empty()
            && let Ok(kp) = load_keypair(trimmed)
        {
            return Ok(kp);
        }
    }
    SuiKeyPair::from_bytes(&raw).map_err(|e| eyre::eyre!("Invalid key file: {}", e))
}

/// Generate a new keypair and print address, public key, and secrets.
pub fn generate(scheme: SignatureScheme, word_length: Option<String>) -> Result<()> {
    let (address, keypair, actual_scheme, mnemonic) =
        generate_new_key(scheme, None, word_length).map_err(|e| eyre::eyre!("{e}"))?;
    println!("Address: {address}");
    println!("Scheme: {:?}", actual_scheme);
    println!("Public key (base64): {}", keypair.public().encode_base64());
    println!("Private key (base64): {}", keypair.encode_base64());
    println!(
        "Private key (bech32): {}",
        keypair.encode().map_err(|e| eyre::eyre!("{e}"))?
    );
    println!("Mnemonic: {mnemonic}");
    Ok(())
}

/// Import a key into the suix keystore file under an alias.
pub fn import_to_keystore(
    keystore_path: Option<PathBuf>,
    secret: &str,
    alias: Option<String>,
) -> Result<()> {
    use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};

    let keypair = load_keypair(secret)?;
    let address = SuiAddress::from(&keypair.public());
    let path = match keystore_path {
        Some(p) => p.to_string_lossy().to_string(),
        None => default_keystore_path()?.to_string_lossy().to_string(),
    };
    let path_buf = PathBuf::from(&path);
    if let Some(parent) = path_buf.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut store = FileBasedKeystore::load_or_create(&path_buf)
            .map_err(|e| eyre::eyre!("Failed to open keystore: {}", e))?;
        store
            .import(alias.clone(), keypair)
            .await
            .map_err(|e| eyre::eyre!("Failed to import key: {}", e))?;
        store
            .save()
            .await
            .map_err(|e| eyre::eyre!("Failed to save keystore: {}", e))?;
        println!("Imported {address} into {path}");
        if let Some(alias) = alias {
            println!("Alias: {alias}");
        }
        Ok(())
    })
}

/// Export a keypair from the suix keystore file.
pub fn export_from_keystore(keystore_path: Option<PathBuf>, address: &str) -> Result<()> {
    use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};

    let address: SuiAddress = address
        .parse()
        .map_err(|e| eyre::eyre!("Invalid address: {}", e))?;
    let path_buf = keystore_path.map_or(default_keystore_path()?, |p| p);
    let store = FileBasedKeystore::load_or_create(&path_buf)
        .map_err(|e| eyre::eyre!("Failed to open keystore: {}", e))?;
    let keypair = store
        .export(&address)
        .map_err(|e| eyre::eyre!("Failed to export key: {}", e))?;
    println!("Address: {address}");
    println!("Public key (base64): {}", keypair.public().encode_base64());
    println!("Private key (base64): {}", keypair.encode_base64());
    println!(
        "Private key (bech32): {}",
        keypair.encode().map_err(|e| eyre::eyre!("{e}"))?
    );
    Ok(())
}

/// List all keys in the suix keystore file.
pub fn list_keystore(keystore_path: Option<PathBuf>) -> Result<()> {
    use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};

    let path_buf = keystore_path.map_or(default_keystore_path()?, |p| p);
    let store = FileBasedKeystore::load_or_create(&path_buf)
        .map_err(|e| eyre::eyre!("Failed to open keystore: {}", e))?;
    let pairs = store.addresses_with_alias();
    if pairs.is_empty() {
        println!("Keystore is empty ({})", path_buf.display());
        return Ok(());
    }
    for (address, alias) in pairs {
        println!(
            "{address}  alias={} pk={}",
            alias.alias, alias.public_key_base64
        );
    }
    Ok(())
}

/// Sign raw bytes with a keypair and print the signature.
pub fn sign_bytes(keypair: &SuiKeyPair, message: &[u8]) -> Result<()> {
    let signature = keypair.sign(message);
    println!("Signer: {}", SuiAddress::from(&keypair.public()));
    println!("Message (hex): {}", hex::encode(message));
    println!("Signature (base64): {}", signature.encode_base64());
    Ok(())
}

/// Sign a transaction intent message with a keypair.
pub fn sign_transaction_data(keypair: &SuiKeyPair, tx_bytes_b64: &str) -> Result<()> {
    use sui_types::transaction::TransactionData;

    let bytes =
        Base64::decode(tx_bytes_b64).map_err(|e| eyre::eyre!("Invalid base64 tx bytes: {}", e))?;
    let tx_data: TransactionData =
        bcs::from_bytes(&bytes).map_err(|e| eyre::eyre!("Invalid TransactionData: {}", e))?;
    let signature = Signature::new_secure(
        &IntentMessage::new(Intent::sui_transaction(), &tx_data),
        keypair,
    );
    println!("Signer: {}", SuiAddress::from(&keypair.public()));
    println!("Tx digest: {}", tx_data.digest());
    println!("Signature (base64): {}", signature.encode_base64());
    Ok(())
}

/// Decode a base64 signature into flag, signature bytes, and public key.
pub fn decode_signature(signature_b64: &str) -> Result<()> {
    let bytes = Base64::decode(signature_b64)
        .map_err(|e| eyre::eyre!("Invalid base64 signature: {}", e))?;
    let signature =
        Signature::from_bytes(&bytes).map_err(|e| eyre::eyre!("Invalid signature bytes: {}", e))?;
    println!("Scheme: {:?}", signature.scheme());
    println!("Signature: {signature:?}");
    Ok(())
}

/// Build a multisig address from public keys, weights, and threshold.
pub fn multisig_address(pks_b64: &[String], weights: &[u8], threshold: u16) -> Result<()> {
    if pks_b64.len() != weights.len() {
        eyre::bail!("Number of public keys must match number of weights");
    }
    let mut pks = Vec::with_capacity(pks_b64.len());
    for pk in pks_b64 {
        pks.push(
            PublicKey::decode_base64(pk).map_err(|e| eyre::eyre!("Invalid public key: {}", e))?,
        );
    }
    let weights: Vec<WeightUnit> = weights.to_vec();
    let multisig_pk =
        MultiSigPublicKey::new(pks, weights, threshold).map_err(|e| eyre::eyre!("{e}"))?;
    let address = SuiAddress::from(&multisig_pk);
    println!("Multisig address: {address}");
    println!("Threshold: {threshold}");
    Ok(())
}

/// Write a keypair to a file in official format.
pub fn write_keypair_file(keypair: &SuiKeyPair, path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    write_keypair_to_file(keypair, path).map_err(|e| eyre::eyre!("{e}"))?;
    println!("Wrote keypair to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scheme() {
        assert!(matches!(
            parse_scheme("ed25519").unwrap(),
            SignatureScheme::ED25519
        ));
        assert!(matches!(
            parse_scheme("secp256k1").unwrap(),
            SignatureScheme::Secp256k1
        ));
        assert!(matches!(
            parse_scheme("secp256r1").unwrap(),
            SignatureScheme::Secp256r1
        ));
        assert!(matches!(
            parse_scheme("r1").unwrap(),
            SignatureScheme::Secp256r1
        ));
        assert!(parse_scheme("sr25519").is_err());
    }

    #[test]
    fn test_keypair_roundtrip() {
        let (_, keypair, _, _) = generate_new_key(SignatureScheme::ED25519, None, None).unwrap();
        let encoded = keypair.encode_base64();
        let loaded = load_keypair(&encoded).unwrap();
        assert_eq!(
            SuiAddress::from(&loaded.public()),
            SuiAddress::from(&keypair.public())
        );
        let bech32 = keypair.encode().unwrap();
        assert!(bech32.starts_with("suiprivkey"));
        let loaded2 = load_keypair(&bech32).unwrap();
        assert_eq!(
            SuiAddress::from(&loaded2.public()),
            SuiAddress::from(&keypair.public())
        );
    }

    #[test]
    fn test_sign_bytes_produces_signature() {
        use fastcrypto::traits::EncodeDecodeBase64;
        let (_, keypair, _, _) = generate_new_key(SignatureScheme::ED25519, None, None).unwrap();
        let signature = keypair.sign(b"hello");
        assert!(!signature.encode_base64().is_empty());
    }
}
