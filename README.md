# AdChain Bidder Database Manager

## NOTES

Must have diesel-cli installed

```shell
sudo apt-get install mysql-server libpq-dev libmysqlclient-dev
cargo install diesel_cli
```

Must have your own .env file containing (DATABASE_URL=mysql://username:password@localhost/acbidder_database)

```shell
echo DATABASE_URL=mysql://username:password@localhost/acbidder_database
```

Must have clean tables for running tests

```shell
diesel migration redo
```

Must change the account and registry address in adchain_registry.rs

Must do tests using 1 thread

```rust
cargo test -- --test-threads=1
```