use clap::{Parser, Subcommand};
use hex;
use std::path::PathBuf;
use zkwasm_host_circuits::host::{datahash::DataHashRecord, mongomerkle::MerkleRecord};

#[derive(Parser)]
#[clap(author, version, about = "CLI tool to check RocksDB key-value pairs")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a key-value pair in a RocksDB database
    CheckRocksDb {
        /// Path to the RocksDB database directory
        #[clap(short, long)]
        db_path: PathBuf,

        /// Target column family to look up in the database, should either be "merkle_records" or "data_records"
        #[clap(short, long)]
        target_cf: String,

        /// Key to look up in the database (hex string like "0A1B2C" or array format like "[10,27,44]")
        #[clap(short, long)]
        key: String,
    },
    CountRocksDb {
        /// Path to the RocksDB database directory
        #[clap(short, long)]
        db_path: PathBuf,

        /// Target column family to look up in the database, should either be "merkle_records" or "data_records"
        #[clap(short, long)]
        target_cf: String,
    },
}

/// Opens a RocksDB database in read-only mode
fn create_read_only_db_handler(rocksdb_path: PathBuf, cf_names: Vec<&str>) -> rocksdb::DB {
    rocksdb::DB::open_cf_for_read_only(&rocksdb::Options::default(), rocksdb_path, cf_names, false)
        .expect("Should be able to open db")
}

/// Parses a key string into a vector of bytes
/// Accepts hex strings (e.g., "0x0A1B2C") or array strings (e.g., "[10,27,44]")
fn parse_key(key_str: &str) -> Result<Vec<u8>, String> {
    if key_str.starts_with('[') && key_str.ends_with(']') {
        // Parse array format: [10,27,44]
        let contents = &key_str[1..key_str.len() - 1];

        // Count how many elements are in the array
        let element_count = contents.split(',').count();

        if element_count == 4 {
            // Parse 4 length array as u64 (could be [u64; 4])
            let values: Result<Vec<u64>, _> = contents
                .split(',')
                .map(|s| {
                    let s = s.trim().replace("_u64", "");
                    s.parse::<u64>()
                })
                .collect();

            if let Ok(u64_values) = values {
                println!("Parsed input as [u64; 4]");
                let mut bytes = Vec::with_capacity(u64_values.len() * 8);
                for val in u64_values {
                    bytes.extend_from_slice(&val.to_le_bytes());
                }
                return Ok(bytes);
            }
        }

        if element_count == 32 {
            println!("Parsed input as [u8; 32]");
        }

        // Parse as regular u8 array
        let values: Result<Vec<u8>, _> = contents
            .split(',')
            .map(|s| s.trim().parse::<u8>())
            .collect();

        values.map_err(|e| format!("Failed to parse array format: {}", e))
    } else {
        // Parse hex string format
        let hex_str = if key_str.starts_with("0x") {
            // Remove "0x" prefix
            &key_str[2..]
        } else {
            key_str
        };

        let bytes =
            hex::decode(hex_str).map_err(|e| format!("Failed to parse hex string: {}", e))?;

        // If the byte length is 32, check if this might be a [u8; 32] or [u64; 4]
        if bytes.len() == 32 {
            println!("Detected 32-byte key (compatible with [u8; 32] or [u64; 4])");
        } else if bytes.len() % 8 == 0 && bytes.len() > 0 {
            println!(
                "Detected {}-byte key ({} u64 values)",
                bytes.len(),
                bytes.len() / 8
            );
        }

        Ok(bytes)
    }
}

const MERKLE_CF_NAME: &str = "merkle_records";
const DATA_CF_NAME: &str = "data_records";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CheckRocksDb {
            db_path,
            target_cf,
            key,
        } => {
            println!("Checking RocksDB at path: {:?}", db_path);

            // Parse the key
            let key_bytes = parse_key(key).map_err(|e| {
                eprintln!("Error parsing key: {}", e);
                e
            })?;

            println!("Looking for key (bytes): {:?}", key_bytes);
            let cf_names = vec![MERKLE_CF_NAME, DATA_CF_NAME];
            // Open the database
            let db = create_read_only_db_handler(db_path.clone(), cf_names);

            let cf = db
                .cf_handle(target_cf)
                .expect("Should be able to get cf handle");
            // Try to get the value
            match db.get_cf(cf, &key_bytes) {
                Ok(Some(value)) => {
                    println!("Key found!");
                    println!("Value (bytes): {:?}", value);

                    // Try to display the value in different formats for convenience
                    if target_cf == MERKLE_CF_NAME {
                        let record = MerkleRecord::from_slice(&value)?;
                        println!("Value (as MerkleRecord): {:?}", record);
                    }

                    if target_cf == DATA_CF_NAME {
                        let record = DataHashRecord::from_slice(&value)?;
                        println!("Value (as DataRecord): {:?}", record);
                    }

                    // Try to interpret as u32 or u64 if appropriate length
                    if value.len() == 4 {
                        let val_u32 = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);
                        println!("Value (as u32, little-endian): {}", val_u32);
                    }
                    if value.len() == 8 {
                        let val_u64 = u64::from_le_bytes([
                            value[0], value[1], value[2], value[3], value[4], value[5], value[6],
                            value[7],
                        ]);
                        println!("Value (as u64, little-endian): {}", val_u64);
                    }

                    // Try to interpret as UTF-8 string
                    match std::str::from_utf8(&value) {
                        Ok(s) => println!("Value (as UTF-8): {}", s),
                        Err(_) => println!("Value is not valid UTF-8"),
                    }
                }
                Ok(None) => {
                    println!("Key not found in the database");
                }
                Err(e) => {
                    eprintln!("Error reading from database: {}", e);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Database error: {}", e),
                    )));
                }
            }
        },
        Commands::CountRocksDb { db_path, target_cf } => {
            println!("Counting RocksDB at path: {:?}", db_path);

            let cf_names = vec![MERKLE_CF_NAME, DATA_CF_NAME];
            // Open the database
            let db = create_read_only_db_handler(db_path.clone(), cf_names);

            let cf = db
                .cf_handle(target_cf)
                .expect("Should be able to get cf handle");

            let iter = db.iterator_cf(cf, rocksdb::IteratorMode::Start);
            
            let count = iter.count();

            println!("Total number of records in column family '{}': {}", target_cf, count);
        },
    }

    Ok(())
}
