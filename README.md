## my-redis

Yet another implementation of redis based on the great [tokio tutorial](https://tokio.rs/tokio/tutorial).
In its current state, the server is limited to single database 
and supports only these [commands](#Commands).

### Quick start guide

To run the server for development purposes, you can execute:
```shell
cargo run --bin server
```

Now you can communicate with the server using [commands](#Commands) via RESP, e.g.:
```shell
cargo run --example set_get
```

### Commands

* GET
* SET