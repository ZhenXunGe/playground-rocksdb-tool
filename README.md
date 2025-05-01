# playground-rocksdb-tool

## Run

### Check for a particular key in a column family

requires

- `--db-path`: path to rocksdb directory
- `--key`: key to query. This should be wrapped by quotes as a string.
- `--target-cf`: target column family to query, either `merkle_records` or `data_records`

```bash
cargo run --release check-rocks-db --db-path /tmp/rocksdb --target-cf merkle_records --key "[1, 2, 3] OR 0x1234567890abcdef"
```

As our hash key is 256 bits, so just need make sure if inputs is "[1, 2, 3, 4]", then 4 len is u64 and 32 len is u8

### Count records in a column family

requires

- `--db-path`: path to rocksdb directory
- `--target-cf`: target column family to query, either `merkle_records` or `data_records`

```bash
cargo run --release count-rocks-db --db-path /tmp/rocksdb --target-cf merkle_records
```

### Flush and Compact specific images `RocksDB`

requires

- `--workspace`: path to `RocksDB` workspace folder.
- `--md5-list`: list of image md5s to perform flush and compact on.
- `--verbose`: if specified, prints additional directory information to stdout during running.

```bash
cargo run --release flush-and-compact --workspace /tmp/rocksdb --md5-list D2144252F3C9DDCA5CA86C23D2EE97E9 6C17A53119CE7FAAD838C22232FBF61A --verbose
```

#### Example: Flush and Compact every image's DB within `RocksDB` workspace

Please see the following [script](scripts/flush_and_compact_all_dbs.sh)

Usage:

```bash
bash scripts/flush_and_compact_all_dbs.sh /dir/to/rocksdb/workspace
```

## Developer Pre-Commit Guide

Developers are recommended to run the following format and lint commands before committing:

```bash
cargo fmt
cargo lint
```
