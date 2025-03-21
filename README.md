# playground-rocksdb-tool

## Run

requires

- --db-path: path to rocksdb directory
- --key: key to query. This should be wrapped by quotes as a string.

```bash
cargo run --release checkrocksdb -- --db-path /tmp/rocksdb --key "[1, 2, 3] OR 0x1234567890abcdef"
```

By default, it will query merkle column family. If you want to query other column family, you can use `--cf-name` option.

```bash
cargo run --release checkrocksdb -- --db-path /tmp/rocksdb --cf-name "default" --key "[1, 2, 3] OR 0x1234567890abcdef"
```
