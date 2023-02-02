# solipsistDB

solipsistDB is an embedded time-series database written in Rust, based on the InfluxDB Line Protocol. It provides sharding, compression and conditional eviction of already consumed data points, making it suitable for deployment in various environments, from cloud to edge.

## Features

Efficient collection, storage, and compression of data.
Sharding of data based on timestamp.
Conditional eviction of consumed data points.
Support for InfluxDB Line Protocol.

## Usage

Each instance of solipsistDB is designed to collect, store, and compress data in an efficient manner. To collect and merge data from multiple solipsistDB instances, use realist. realist reads data from multiple solipsistDB instances, merges them, compresses and indexes the data, making it possible to run queries and analysis against the collected data.

There are multiple different modes to get the data from solipsistDB to realist:

Regular timed batches
On-demand
Real-time, where solipsistDB instances send new data via MQTT to realist

## Installation

To install solipsistDB, add the following to your Cargo.toml:

```toml
[dependencies]
solipsistDB = "0.1.0"
```

## Example

```rust
use solipsistDB::{SolipsistDB, Config};

let config = Config {
    cwd: Path::new("./db_path")
};

let solipsist_db = SolipsistDB::new(config);

solipsist_db.write("temperature,location=office temperature=72.5 1465839830100400200")
    .expect("Failed to write data to solipsistDB");

let query_result = solipsist_db.query("SELECT temperature FROM temperature WHERE location = 'office'")
    .expect("Failed to query solipsistDB");

println!("{:?}", query_result);
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

solipsistDB is released under the MIT license.

## Acknowledgements

solipsistDB uses the following open-source libraries:

- `lz4` for data compression.
- `ntp` for syncing clocks to later merge data from multiple instances

## Additional information

realist is a separate tool that is meant to collect and merge data from multiple solipsistDB instances, it is not included in this project.