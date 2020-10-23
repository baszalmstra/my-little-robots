# My Little Robots

My Little Robots is a programming game. 

## Running the backend

Install the following dependencies: 

```
cargo install systemfd cargo-watch
```

Start the backend with:

```
cd backend
systemfd --no-pid -s http::3030 -- cargo watch -w src -w frontend/src -x run
```

This will listen for any changes in the `backend` crate and recompile and 
restart the server on success.
