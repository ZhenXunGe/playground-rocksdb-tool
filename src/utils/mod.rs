use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use zkwasm_host_circuits::host::db::RocksDB;

use crate::constants::ROCKSDB_MAX_OPEN_FILES;

pub struct CaseInsensitiveMD5(String);

impl CaseInsensitiveMD5 {
    pub fn new(md5: &str) -> Self {
        CaseInsensitiveMD5(md5.to_uppercase())
    }
}

impl Display for CaseInsensitiveMD5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / 1_048_576.0
}

fn get_hex_split_md5_workspace_path(md5: &CaseInsensitiveMD5, workspace: PathBuf) -> PathBuf {
    let str = md5.to_string();
    workspace.join("images").join(&str[0..2]).join(&str[2..4]).join(str)
}

fn delete_old_logs(rocksdb_dir: PathBuf, md5: &CaseInsensitiveMD5) -> std::io::Result<()> {
    let dir = get_hex_split_md5_workspace_path(md5, rocksdb_dir);
    for entry in std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_str().map_or(false, |name| name.starts_with("LOG.old.")))
    {
        std::fs::remove_file(entry.path())?;
    }

    Ok(())
}

fn get_directory_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0;
    if !path.is_dir() {
        return Ok(size);
    }
    for entry_result in std::fs::read_dir(path)? {
        let entry = entry_result?;
        let metadata = entry.metadata()?;
        let entry_path = entry.path();

        if metadata.is_dir() {
            size += get_directory_size(&entry_path)?;
        } else {
            size += metadata.len();
        }
    }
    Ok(size)
}

fn sst_files_count(path: &Path) -> std::io::Result<usize> {
    Ok(std::fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "sst"))
        .count())
}

fn print_dir_contents(rocksdb_dir: PathBuf, md5: &CaseInsensitiveMD5) {
    let dir = get_hex_split_md5_workspace_path(md5, rocksdb_dir);
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!("ls -alh {}", dir.display()))
        .output()
        .expect("Failed to execute command");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!(
        "Size of directory {}",
        bytes_to_mb(get_directory_size(dir.as_path()).expect("Should get size"))
    );
    println!(
        "Num of sst files {}",
        sst_files_count(dir.as_path()).expect("Should get sst files")
    );
}

// fn flush_and_compact_specific_rocksdb() {
//     std::env::set_var("ROCKSDB_FOLDER", "rocksdb");
//     let md5 = CaseInsensitiveMD5::new(
//         &std::env::var("FLUSH_AND_COMPACT_IMAGE_MD5")
//             .unwrap_or_else(|_| DEFAULT_IMAGE_MD5.to_string()),
//     );
//
//     println!("Starting flush and compact {md5}");
//     print_dir_contents(&md5);
//
//     let start = std::time::Instant::now();
//     {
//         let handler = RocksDBHandler::create_db_handler(&md5, false)
//             .expect("RocksDB handler must be available in write mode, stop the dry run server");
//
//         handler.print_stats();
//         flush_and_compact(&handler).expect("Should flush and compact");
//         handler.print_stats();
//     }
//     let dur = start.elapsed().as_secs_f64();
//
//     delete_old_logs(&md5).expect("Should delete");
//
//     println!("Finished flush and compact, took {dur} seconds");
//     print_dir_contents(&md5);
// }

pub struct RocksDBHandler;

impl RocksDBHandler {
    pub fn build_handler_options() -> rocksdb::Options {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(ROCKSDB_MAX_OPEN_FILES);
        opts
    }

    pub fn create_db_handler(
        rocksdb_dir: PathBuf,
        md5: &CaseInsensitiveMD5,
        read_only: bool,
    ) -> anyhow::Result<RocksDB> {
        let dir = get_hex_split_md5_workspace_path(md5, rocksdb_dir);
        let opts = Self::build_handler_options();
        if read_only {
            RocksDB::new_read_only_with_options(dir, opts.clone(), opts.clone(), opts.clone())
        } else {
            RocksDB::new_with_options(dir, opts.clone(), opts.clone(), opts.clone())
        }
        .map_err(|e| {
            anyhow::anyhow!(
                "Create {} RocksDB handler error: {e}",
                if read_only { "read-only" } else { "write" }
            )
        })
    }
}
