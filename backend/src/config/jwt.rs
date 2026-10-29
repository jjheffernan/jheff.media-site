use data_encoding::HEXLOWER;
use log::{error, info};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static JWT_KEY: OnceLock<[u8; 32]> = OnceLock::new();

const DEV_SECRET_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/secret.key");
const ENV_SECRET: &str = "YEW_FULLSTACK_JWT_SECRET";
const ENV_SECRET_FILE: &str = "YEW_FULLSTACK_JWT_SECRET_FILE";

fn read_key_file(path: &str) -> [u8; 32] {
    let bytes = fs::read(path).unwrap_or_else(|e| {
        error!("Failed to read JWT secret file {}: {}", path, e);
        std::process::exit(1);
    });
    if bytes.len() != 32 {
        error!(
            "JWT secret file {} must be exactly 32 bytes, got {}",
            path,
            bytes.len()
        );
        std::process::exit(1);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    key
}

fn decode_hex_secret(value: &str) -> [u8; 32] {
    let bytes = HEXLOWER.decode(value.trim().as_bytes()).unwrap_or_else(|e| {
        error!(
            "{} must be 64 hex characters (32 bytes): {}",
            ENV_SECRET, e
        );
        std::process::exit(1);
    });
    if bytes.len() != 32 {
        error!(
            "{} must decode to 32 bytes, got {}",
            ENV_SECRET,
            bytes.len()
        );
        std::process::exit(1);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    key
}

fn load_key() -> [u8; 32] {
    if let Ok(path) = env::var(ENV_SECRET_FILE) {
        if !path.is_empty() {
            info!("Loading JWT secret from {}", ENV_SECRET_FILE);
            return read_key_file(&path);
        }
    }
    if let Ok(value) = env::var(ENV_SECRET) {
        if !value.is_empty() {
            info!("Loading JWT secret from {}", ENV_SECRET);
            return decode_hex_secret(&value);
        }
    }
    if Path::new(DEV_SECRET_PATH).exists() {
        info!("Loading JWT secret from dev file {}", DEV_SECRET_PATH);
        return read_key_file(DEV_SECRET_PATH);
    }
    error!(
        "JWT secret not configured: set {} or {}, or create {}",
        ENV_SECRET_FILE, ENV_SECRET, DEV_SECRET_PATH
    );
    std::process::exit(1);
}

pub fn key() -> &'static [u8; 32] {
    JWT_KEY.get_or_init(load_key)
}

pub fn init() {
    let _ = key();
}
