# playground-rocksdb-tool

## Run

requires

- --db-path: path to rocksdb directory
- --key: key to query. This should be wrapped by quotes as a string.
- --target-cf: target column family to query, either `merkle_records` or `data_records`

```bash
cargo run --release check-rocks-db --db-path /tmp/rocksdb --target-cf merkle_records --key "[1, 2, 3] OR 0x1234567890abcdef"
```

 As our hash key is 256 bits, so just need make sure if inputs is "[1, 2, 3, 4]", then 4 len is u64 and 32 len is u8
