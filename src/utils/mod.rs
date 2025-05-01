use std::fmt::Display;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;
use zkwasm_host_circuits::host::db::RocksDB;

use crate::constants::ROCKSDB_MAX_OPEN_FILES;

pub struct CaseInsensitiveMD5(String);

impl CaseInsensitiveMD5 {
    pub fn new<P: AsRef<str>>(md5: P) -> Self {
        CaseInsensitiveMD5(md5.as_ref().to_uppercase())
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

fn get_hex_split_md5_workspace_path(workspace: &Path, md5: &CaseInsensitiveMD5) -> PathBuf {
    let str = md5.to_string();
    workspace.join("images").join(&str[0..2]).join(&str[2..4]).join(str)
}

fn sst_files_count(path: &Path) -> std::io::Result<usize> {
    Ok(std::fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "sst"))
        .count())
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

        size += if metadata.is_dir() {
            get_directory_size(&entry_path)?
        } else {
            metadata.len()
        };
    }
    Ok(size)
}

pub fn delete_old_logs(workspace: &Path, md5: &CaseInsensitiveMD5) -> std::io::Result<()> {
    fn check_name<F: FnOnce(&str) -> bool>(entry: &DirEntry, check_fn: F) -> bool {
        entry.file_name().to_str().is_some_and(check_fn)
    }

    let dir = get_hex_split_md5_workspace_path(workspace, md5);
    for entry in std::fs::read_dir(dir)?.filter_map(Result::ok).filter(|entry| {
        check_name(entry, |n| n.starts_with("LOG.old."))
            || (check_name(entry, |n| n.ends_with(".log")) && entry.metadata().is_ok_and(|md| md.len() == 0))
    }) {
        std::fs::remove_file(entry.path())?;
    }
    Ok(())
}

pub fn print_dir_contents(
    workspace: &Path,
    md5: &CaseInsensitiveMD5,
    print_path: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    let dir = get_hex_split_md5_workspace_path(workspace, md5);
    if print_path {
        println!("\tDirectory: {}", dir.display());
    }
    if !dir.exists() {
        return Err(anyhow::anyhow!("Directory doesn't exist {dir:?}"));
    }
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!("ls -alh {}", dir.display()))
        .output()
        .expect("Failed to execute command");
    println!(
        "\tNum of sst files: {}\n\tSize of directory: {:.6}MB",
        sst_files_count(dir.as_path()).map_err(|e| anyhow::anyhow!("Failed getting sst file count, {e}"))?,
        bytes_to_mb(
            get_directory_size(dir.as_path()).map_err(|e| anyhow::anyhow!("Failed getting directory size, {e}"))?
        ),
    );
    if verbose {
        println!(
            "\tDirectory contents: {}",
            String::from_utf8_lossy(&output.stdout)
                .trim_end_matches('\n')
                .replace('\n', "\n\t\t")
        );
    }
    Ok(())
}

pub struct RocksDBHandler;

impl RocksDBHandler {
    fn build_handler_options() -> rocksdb::Options {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(ROCKSDB_MAX_OPEN_FILES);
        opts
    }

    pub fn create_db_handler(workspace: &Path, md5: &CaseInsensitiveMD5, read_only: bool) -> anyhow::Result<RocksDB> {
        let dir = get_hex_split_md5_workspace_path(workspace, md5);
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
