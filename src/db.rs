pub fn create_prover_dry_run_db_handler(rocksdb_path: PathBuf) -> rocksdb::DB {
    rocksdb::DB::open_for_read_only(&rocksdb::Options::default(), rocksdb_path, false)
    .expect("Should be able to open db")
}
