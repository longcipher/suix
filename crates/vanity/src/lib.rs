use std::{
    collections::HashMap,
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use eyre::{Context, Result, bail};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use sui_keys::keypair_file::write_keypair_to_file;
use sui_types::crypto::{EncodeDecodeBase64, SignatureScheme, SuiKeyPair};

const DEFAULT_ADDRESSES_PER_ROUND: usize = 10000;

/// Configuration for vanity address generation
#[derive(Debug, Clone)]
pub struct VanityConfig {
    pub starts_with: Option<String>,
    pub ends_with: Option<String>,
    pub contains: Option<String>,
    pub scheme: SignatureScheme,
    pub save_path: Option<String>, // None means print to terminal only
    pub threads: usize,
    pub max_addresses: usize,
    pub addresses_per_round: usize,
}

impl Default for VanityConfig {
    fn default() -> Self {
        Self {
            starts_with: None,
            ends_with: None,
            contains: None,
            scheme: SignatureScheme::ED25519,
            save_path: None, // Default to terminal output
            threads: 0,      // 0 means use default (number of cores)
            max_addresses: 1,
            addresses_per_round: DEFAULT_ADDRESSES_PER_ROUND,
        }
    }
}

/// A generated key pair with its address
#[derive(Debug)]
pub struct GeneratedKey {
    pub address: String,
    pub keypair: SuiKeyPair,
}

/// Convert string to hexspeak using similar-looking hex characters
fn string_to_hexspeak(s: &str) -> Result<Vec<u8>> {
    let hexspeak_map: HashMap<char, char> = [
        ('a', 'a'),
        ('b', 'b'),
        ('c', 'c'),
        ('d', 'd'),
        ('e', 'e'),
        ('f', 'f'),
        ('g', '9'),
        ('i', '1'),
        ('j', '1'),
        ('l', '1'),
        ('o', '0'),
        ('q', '9'),
        ('s', '5'),
        ('t', '7'),
        ('z', '2'),
        ('0', '0'),
        ('1', '1'),
        ('2', '2'),
        ('3', '3'),
        ('4', '4'),
        ('5', '5'),
        ('6', '6'),
        ('7', '7'),
        ('8', '8'),
        ('9', '9'),
    ]
    .iter()
    .cloned()
    .collect();

    let lowercase_s = s.to_lowercase();
    let mut hex_string = String::new();

    for c in lowercase_s.chars() {
        if let Some(&hex_char) = hexspeak_map.get(&c) {
            hex_string.push(hex_char);
        } else {
            bail!("Cannot map character '{}' to hexspeak", c);
        }
    }

    hex_str_to_bytes(&hex_string)
}

/// Convert hex string to bytes
fn hex_str_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
    let hex_str = hex_str.trim_start_matches("0x");
    let mut bytes = Vec::with_capacity(hex_str.len().div_ceil(2));

    for i in (0..hex_str.len()).step_by(2) {
        let end = std::cmp::min(i + 2, hex_str.len());
        let hex_pair = &hex_str[i..end];

        let padded_hex = if hex_pair.len() == 1 {
            format!("{hex_pair}0")
        } else {
            hex_pair.to_string()
        };

        bytes.push(
            u8::from_str_radix(&padded_hex, 16)
                .wrap_err_with(|| format!("Invalid hex string: {padded_hex}"))?,
        );
    }

    Ok(bytes)
}

/// Parse pattern (regex or hex string) into searchable format
fn parse_pattern(pattern: &str) -> Result<(Vec<u8>, Option<u8>, Option<Regex>)> {
    // Try to parse as hex string first
    if pattern.starts_with("0x") {
        let mut needle = hex_str_to_bytes(pattern)?;
        let uneven_nibble = if pattern.len() % 2 == 1 {
            needle.pop()
        } else {
            None
        };
        return Ok((needle, uneven_nibble, None));
    }

    // Try to parse as regex if it contains regex special characters
    if (pattern.contains('^')
        || pattern.contains('$')
        || pattern.contains('[')
        || pattern.contains(']')
        || pattern.contains('(')
        || pattern.contains(')')
        || pattern.contains('|')
        || pattern.contains('{')
        || pattern.contains('}')
        || pattern.contains('+')
        || pattern.contains('*')
        || pattern.contains('?'))
        && let Ok(regex) = Regex::new(pattern)
    {
        return Ok((Vec::new(), None, Some(regex)));
    }

    // Fall back to hexspeak conversion
    let mut needle = string_to_hexspeak(pattern)?;
    let uneven_nibble = if pattern.len() % 2 == 1 {
        needle.pop()
    } else {
        None
    };

    Ok((needle, uneven_nibble, None))
}

/// Generate a new key pair and address using Sui official libraries
fn generate_new_key_with_scheme(scheme: SignatureScheme) -> Result<GeneratedKey> {
    let (address, keypair, _scheme, _seed) =
        sui_keys::key_derive::generate_new_key(scheme, None, None)
            .map_err(|e| eyre::eyre!("Failed to generate key: {}", e))?;

    // Convert SuiAddress to hex string
    let address_str = format!("{address}");

    Ok(GeneratedKey {
        address: address_str,
        keypair,
    })
}

/// Generate a new ED25519 key pair (kept for backwards compatibility in tests).
#[allow(dead_code)]
fn generate_new_key() -> Result<GeneratedKey> {
    generate_new_key_with_scheme(SignatureScheme::ED25519)
}

/// Estimate the expected number of trials for a hex pattern of `nibbles` length.
/// Each nibble has a 1/16 chance, so expected trials = 16^nibbles.
pub fn estimate_difficulty(nibbles: usize) -> f64 {
    16f64.powi(nibbles as i32)
}

/// Count effective hex nibbles in a pattern for difficulty estimation.
pub fn pattern_nibbles(pattern: &str) -> Option<usize> {
    if pattern.starts_with("0x") {
        Some(pattern.trim_start_matches("0x").len())
    } else if pattern.contains('^')
        || pattern.contains('$')
        || pattern.contains('[')
        || pattern.contains(']')
        || pattern.contains('(')
        || pattern.contains(')')
        || pattern.contains('|')
        || pattern.contains('{')
        || pattern.contains('}')
        || pattern.contains('+')
        || pattern.contains('*')
        || pattern.contains('?')
    {
        None
    } else {
        Some(pattern.len())
    }
}

/// Match position within the address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchPosition {
    Prefix,
    Suffix,
    Contains,
}

/// Check if address matches the pattern
fn matches_pattern(
    address: &str,
    needle: &[u8],
    uneven_nibble: Option<u8>,
    regex: &Option<Regex>,
    position: MatchPosition,
) -> bool {
    if let Some(re) = regex {
        return re.is_match(address);
    }

    // Remove 0x prefix for byte comparison
    let address_hex = if let Some(stripped) = address.strip_prefix("0x") {
        stripped
    } else {
        address
    };

    // Convert address string to bytes for comparison
    let address_bytes = match hex::decode(address_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let matches_bytes = match position {
        MatchPosition::Prefix => address_bytes.starts_with(needle),
        MatchPosition::Suffix => address_bytes.ends_with(needle),
        MatchPosition::Contains => address_bytes
            .windows(needle.len().max(1))
            .any(|w| w == needle),
    };

    if !matches_bytes {
        return false;
    }

    if position == MatchPosition::Contains {
        return true;
    }

    if let Some(uneven) = uneven_nibble {
        let relevant_byte_idx = if position == MatchPosition::Prefix {
            needle.len()
        } else {
            address_bytes.len() - needle.len() - 1
        };

        if relevant_byte_idx < address_bytes.len() {
            let relevant_nibble = if position == MatchPosition::Prefix {
                address_bytes[relevant_byte_idx] & 0xf0
            } else {
                address_bytes[relevant_byte_idx] & 0x0f
            };
            return relevant_nibble == uneven;
        }
    }

    true
}

/// Save key to file using Sui official format
fn save_key_to_file(key: &GeneratedKey, output_dir: &Path) -> Result<()> {
    let address_clean = if key.address.starts_with("0x") {
        &key.address[2..]
    } else {
        &key.address
    };
    let filename = format!("{address_clean}.key");
    let filepath = output_dir.join(filename);

    write_keypair_to_file(&key.keypair, &filepath)
        .map_err(|e| eyre::eyre!("Failed to write keypair file: {}", e))?;

    Ok(())
}

/// Generate vanity addresses based on configuration
pub fn generate_vanity_addresses(config: &VanityConfig) -> Result<()> {
    // Set up thread pool
    let thread_count = if config.threads == 0 {
        rayon::current_num_threads()
    } else {
        config.threads
    };

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .context("Failed to build thread pool")?;

    // Parse patterns
    let starts_pattern = if let Some(ref pattern) = config.starts_with {
        Some(parse_pattern(pattern).context("Failed to parse starts-with pattern")?)
    } else {
        None
    };

    let ends_pattern = if let Some(ref pattern) = config.ends_with {
        Some(parse_pattern(pattern).context("Failed to parse ends-with pattern")?)
    } else {
        None
    };

    let contains_pattern = if let Some(ref pattern) = config.contains {
        Some(parse_pattern(pattern).context("Failed to parse contains pattern")?)
    } else {
        None
    };

    let count = AtomicUsize::new(0);
    let mut tried = 0;

    println!("Generating vanity addresses with {thread_count} threads...");
    println!("Scheme: {:?}", config.scheme);
    if let Some(ref pattern) = config.starts_with {
        println!("Starts with: {pattern}");
        if let Ok((needle, uneven, regex)) = parse_pattern(pattern) {
            if let Some(re) = &regex {
                println!("  Using regex: {}", re.as_str());
            } else {
                println!("  Looking for bytes: {needle:02x?}");
                if let Some(nibble) = uneven {
                    println!("  Plus uneven nibble: {nibble:02x}");
                }
            }
        }
    }
    if let Some(ref pattern) = config.ends_with {
        println!("Ends with: {pattern}");
        if let Ok((needle, uneven, regex)) = parse_pattern(pattern) {
            if let Some(re) = &regex {
                println!("  Using regex: {}", re.as_str());
            } else {
                println!("  Looking for bytes: {needle:02x?}");
                if let Some(nibble) = uneven {
                    println!("  Plus uneven nibble: {nibble:02x}");
                }
            }
        }
    }
    if let Some(ref pattern) = config.contains {
        println!("Contains: {pattern}");
        if let Ok((needle, uneven, regex)) = parse_pattern(pattern) {
            if let Some(re) = &regex {
                println!("  Using regex: {}", re.as_str());
            } else {
                println!("  Looking for bytes: {needle:02x?}");
                if let Some(nibble) = uneven {
                    println!("  Plus uneven nibble: {nibble:02x}");
                }
            }
        }
        if let Some(nibbles) = pattern_nibbles(pattern) {
            println!(
                "  Estimated trials: ~{:.0} (16^{nibbles})",
                estimate_difficulty(nibbles)
            );
        }
    }
    println!("Target: {} addresses", config.max_addresses);
    println!();

    let scheme = config.scheme;
    pool.install(|| {
        while count.load(Ordering::Relaxed) < config.max_addresses {
            (0..config.addresses_per_round)
                .into_par_iter()
                .for_each(|_| {
                    if count.load(Ordering::Relaxed) >= config.max_addresses {
                        return;
                    }

                    let key = match generate_new_key_with_scheme(scheme) {
                        Ok(key) => key,
                        Err(_) => return,
                    };

                    let mut matches = true;

                    // Check starts-with pattern
                    if let Some((ref needle, uneven_nibble, ref regex)) = starts_pattern
                        && !matches_pattern(
                            &key.address,
                            needle,
                            uneven_nibble,
                            regex,
                            MatchPosition::Prefix,
                        )
                    {
                        matches = false;
                    }

                    // Check ends-with pattern
                    if matches
                        && let Some((ref needle, uneven_nibble, ref regex)) = ends_pattern
                        && !matches_pattern(
                            &key.address,
                            needle,
                            uneven_nibble,
                            regex,
                            MatchPosition::Suffix,
                        )
                    {
                        matches = false;
                    }

                    // Check contains pattern
                    if matches
                        && let Some((ref needle, uneven_nibble, ref regex)) = contains_pattern
                        && !matches_pattern(
                            &key.address,
                            needle,
                            uneven_nibble,
                            regex,
                            MatchPosition::Contains,
                        )
                    {
                        matches = false;
                    }

                    if matches {
                        // Use compare_exchange to avoid race condition
                        let current = count.load(Ordering::Relaxed);
                        if current >= config.max_addresses {
                            return;
                        }

                        // Try to increment count atomically
                        if count
                            .compare_exchange_weak(
                                current,
                                current + 1,
                                Ordering::Relaxed,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            let address_clean = if key.address.starts_with("0x") {
                                &key.address[2..]
                            } else {
                                &key.address
                            };

                            if let Some(ref save_path) = config.save_path {
                                // Save to file
                                let output_path = Path::new(save_path);
                                if let Err(e) = save_key_to_file(&key, output_path) {
                                    eprintln!("Failed to save key: {e}");
                                    return;
                                }

                                println!(
                                    "Found match {}/{}: {} -> {}/{}.key",
                                    current + 1,
                                    config.max_addresses,
                                    key.address,
                                    save_path,
                                    address_clean
                                );
                            } else {
                                // Print to terminal
                                println!("Found match {}/{}:", current + 1, config.max_addresses);
                                println!("Address: {}", key.address);

                                // Convert keypair to base64 string for terminal output
                                let encoded_key = key.keypair.encode_base64();
                                println!("Private Key: {encoded_key}");
                                println!();
                            }
                        }
                    }
                });

            tried += config.addresses_per_round;
            let current_count = count.load(Ordering::Relaxed);
            if current_count < config.max_addresses {
                print!("\rTried: {tried} addresses, found: {current_count}");
                io::stdout().flush().ok();
            }
        }
    });

    println!(
        "\nCompleted! Generated {} vanity addresses{}.",
        count.load(Ordering::Relaxed),
        if config.save_path.is_some() {
            " and saved to files"
        } else {
            ""
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_hexspeak() {
        let result = string_to_hexspeak("ace").unwrap();
        assert_eq!(result, vec![0xac, 0xe0]);
    }

    #[test]
    fn test_hex_str_to_bytes() {
        let result = hex_str_to_bytes("0xabcd").unwrap();
        assert_eq!(result, vec![0xab, 0xcd]);
    }

    #[test]
    fn test_hex_str_to_bytes_odd_length() {
        let result = hex_str_to_bytes("abc").unwrap();
        assert_eq!(result, vec![0xab, 0xc0]);
    }

    #[test]
    fn test_generate_new_key() {
        let key = generate_new_key().unwrap();
        // Sui addresses are 66 characters: "0x" + 64 hex chars (32 bytes)
        assert_eq!(key.address.len(), 66);
        assert!(key.address.starts_with("0x"));
    }

    #[test]
    fn test_matches_contains() {
        let needle = vec![0xab, 0xcd];
        assert!(matches_pattern(
            "0x00abcdef00",
            &needle,
            None,
            &None,
            MatchPosition::Contains
        ));
        assert!(!matches_pattern(
            "0x0011223300",
            &needle,
            None,
            &None,
            MatchPosition::Contains
        ));
    }

    #[test]
    fn test_matches_prefix_suffix() {
        let needle = vec![0xab];
        assert!(matches_pattern(
            "0xab00112200",
            &needle,
            None,
            &None,
            MatchPosition::Prefix
        ));
        assert!(!matches_pattern(
            "0x00ab112200",
            &needle,
            None,
            &None,
            MatchPosition::Prefix
        ));
        assert!(matches_pattern(
            "0x00112200ab",
            &needle,
            None,
            &None,
            MatchPosition::Suffix
        ));
    }

    #[test]
    fn test_estimate_difficulty() {
        assert_eq!(estimate_difficulty(0), 1.0);
        assert_eq!(estimate_difficulty(1), 16.0);
        assert_eq!(estimate_difficulty(2), 256.0);
        assert_eq!(pattern_nibbles("0xabcd"), Some(4));
        assert_eq!(pattern_nibbles("ace"), Some(3));
        assert_eq!(pattern_nibbles("^[a-f]{4}"), None);
    }

    #[test]
    fn test_generate_secp256k1() {
        let key = generate_new_key_with_scheme(SignatureScheme::Secp256k1).unwrap();
        assert_eq!(key.address.len(), 66);
        let key = generate_new_key_with_scheme(SignatureScheme::Secp256r1).unwrap();
        assert_eq!(key.address.len(), 66);
    }
}
