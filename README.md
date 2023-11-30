# Rust Shamir Secret Sharing Server

Current functionality:
- (WIP) Shamir Secret Sharing

## Build and run

In this example we'll spin up 3 parties. 

### Create parties
Run 3 parties on ports 8080, 8081 and 8082 by running respectively:

```
cargo run <port>
```

### Trigger sharing amongst all parties

Trigger each party to send a share to the given other parties (ports). For example, for the party running on port 8080: 
```
echo "COMMUNICATE_SHARES 8081 8082" | nc 127.0.0.1 8080
```

This will make party listening on port 8080 send the command "RECEIVE_SHARE x" to parties listening on ports 8081 and 8082. Those parties will store those shares in their local storage. 

Similarly, trigger 8081 and 8082 to send their shares.
```
echo "COMMUNICATE_SHARES 8080 8082" | nc 127.0.0.1 8081
echo "COMMUNICATE_SHARES 8080 8081" | nc 127.0.0.1 8082
```


### (For testpurposes) Print Shares

This is (temporarily) available for testing purposes; trigger a party to print the shares they received:
```
echo "SHOW_SHARES"| nc 127.0.0.1 8081
```


