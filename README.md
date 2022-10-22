# eats sats

A basic payment page which connects to LND, written in Rust using Rocket.rs.

Edit the `config.cfg` with your `lnd` lightning node details and run with `cargo run`.

If you need to connect to a remote node you could use:
```
ssh user@my_node_ip -q -N -L 10009:localhost:10009
```

## TODO
bring back coreln :'(

![image](https://user-images.githubusercontent.com/24557779/194470873-cad343d2-6841-45b1-9bfa-9053b784f1e7.png)
