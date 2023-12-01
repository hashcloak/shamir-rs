# Rust Shamir Secret Sharing Server

Current functionality:
- (WIP) Shamir Secret Sharing

## Build and run

In this example we'll spin up 3 parties. 

### Create parties
Run 3 parties on ports 8080, 8081 and 8082 by running in 3 terminals respectively:

```
cargo run 1 "1:8080" "2:8081" "3:8082"

cargo run 2 "1:8080" "2:8081" "3:8082"

cargo run 3 "1:8080" "2:8081" "3:8082"
```


### Trigger sharing amongst all parties

Trigger each party to send a share to the given other parties (ports). For example, for the party running on port 8080: 
```
echo "COMMUNICATE_SHARES" | nc 127.0.0.1 8080
```

This will make party listening on port 8080 send the command "RECEIVE_SHARE x" to parties listening on ports 8081 and 8082. Those parties will store those shares in their local storage. 

Similarly, trigger 8081 and 8082 to send their shares.
```
echo "COMMUNICATE_SHARES" | nc 127.0.0.1 8081
echo "COMMUNICATE_SHARES" | nc 127.0.0.1 8082
```

### Trigger Sum calculation and distribution

```
echo "SUM_AND_DISTRIBUTE" | nc 127.0.0.1 8080
echo "SUM_AND_DISTRIBUTE" | nc 127.0.0.1 8081
echo "SUM_AND_DISTRIBUTE" | nc 127.0.0.1 8082
```

### Trigger calculation of result, print to console

This must give the same result and be equal to the sum of the initial secrets!

```
echo "GIVE_RESULT"| nc 127.0.0.1 8080 
echo "GIVE_RESULT"| nc 127.0.0.1 8081
echo "GIVE_RESULT"| nc 127.0.0.1 8082
```

### (For testpurposes) Print Shares

This is (temporarily) available for testing purposes; trigger a party to print the shares they received:
```
echo "SHOW_SHARES"| nc 127.0.0.1 8081
```
