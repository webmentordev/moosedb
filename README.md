# 🚧 Under Development

> ⚠️ Do not use this system yet. It is still under active development.  

# Build Instructions

This project requires the latest versions of Node.js and Rust.

## Development Mode

Execute the `start.sh` script to run the application in development mode.

## Production Build

Execute the `build.sh` script to build the project for release. This will create a binary file with the backend (Actix) and frontend (Nuxt's dist folder) embedded into it, which can then be run as a standalone application.


## Default Login Credentials

Use the following credentials to log in for the first time:

- **Email:** admin@moosedb.com  
- **Password:** moosedb

> ⚠️ For security reasons, make sure to change the default email and password after your first login.


## Performance

> 🖥️ Tested on Core i7 @12700KF, without I/O file logging.
```
ahmer@zoro:~$ wrk -t4 -c1000 -d10s http://127.0.0.1:8855/
Running 10s test @ http://127.0.0.1:8855/
  4 threads and 1000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.69ms    1.79ms  52.71ms   75.42%
    Req/Sec    86.75k    10.45k  133.78k    77.89%
  3455913 requests in 10.10s, 0.86GB read
Requests/sec: 342211.18
Transfer/sec:     87.14MB
```