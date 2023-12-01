# Rust Shamir Secret Sharing Server

In the Hachathon Q3 at HashCloak, and for learning purposes, we decided to explore the networking process behind MPC protocols. To achieve this goal, we implemented some functionalities of Shamir secret-sharing, including certain networking aspects using TCP sockets. These functionalities encompass sharing distribution, secure addition of secrets, and the reconstruction of secrets given a set of shares. In this implementation, we are considering multiple parties—specifically, the examples shown below are executed using three parties. At the moment, the input of the parties is generated at random instead of being provided by the parties themselves. Additionally, the actions of the parties are triggered by an external user who sends a message to the listening port of the corresponding party to perform a certain action.

While this is not production-quality source code, we managed to grasp basic concepts of the tokio library and some of the underlying details and difficulties that come with implementing a distributed network.

## Build and run

In this example we'll spin up 3 parties and show how to perform share distribution, the protocol for addition, and the protocol for reconstructing a secret.

### Create parties

To create 3 parties on ports 8080, 8081 and 8082, you should spawn 3 terminals and run the following commands in each terminal respectively:

```
cargo run 1 "1:8080" "2:8081" "3:8082"

cargo run 2 "1:8080" "2:8081" "3:8082"

cargo run 3 "1:8080" "2:8081" "3:8082"
```

While you execute those commands, the parties wait for the other parties to be ready. If you execute `cargo run 1 "1:8080" "2:8081" "3:8082"`, you are telling to the program that you are the party with ID 1 such that you are listening in the port 8080, and connecting with other two parties with ID 2 in the port 8081, and with ID 3 in the port 8082. All the connections are done in `localhost`.

### Trigger sharing amongst all parties

The following command triggers each party to send a share to the given other parties (ports). For example, for the party running on port 8080: 

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
