# HIVE

An distributed state log machine based on RAFT protocol

- Leader Election: A new leader must be choosen when an existing leader fails
- Log Replication: The leader must accept log entries from clients and replicate them across the cluster, forcing the other logs to agree with its own
- Safety: If any server has applied a particular log entry to its state machine, then no other server may apply a different command for the same log index

### Running

- In one terminal executes:

```sh
cargo run
```

This command should output the libp2p address of the running instance

- In the other terminal executes:

```sh
cargo run -- {libp2p_address}
```

This command will running another instance connected to the previous one
