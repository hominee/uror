# Intro
Uror is light-weight and blazing-fast service for url-shorting, uri-obfuscation 

# API
- `post`/`get` request with `uri` params to set the short url for uri, and return this short url. There are some way to pass the `uri` params:

  - `curl -x POST http://127.0.0.1:7856/ -H '{"content-type": "application/json"}' -d '{"uri": "https://example.com"}' ` 

  - `curl -x POST http://127.0.0.1:7856/ -H '{"content-type": "application/x-www-form-urlencoded"}' -d 'uri=https://example.com' ` 

  - `curl -x GET http://127.0.0.1:7856/?uri=https://example.com `

- `get` request end with short uri, directly redirect to the target uri 
	Simply put the shot url as sub-path of the request  

  - `curl -x GET http://127.0.0.1:7856/3LlaM1XoNYum`

---

The url-shorting is enabled by default, if you wanna obfuscate uris, not shorten the uri, recompile the project instead.
```bash 
// obfuscating url without logging 
cargo build --release --features obfs
// obfuscating url with logging 
cargo build --release --features obfs2 

// shorting url without logging 
cargo build --release 
// shorting url with logging 
cargo build --release --features logger
```
# Configuration

There are some options you can set according to your requirement.
- `DATABASE_URL`: the path to local database e.g. `uri.db`
- `ADDR`: the host and port to bind with, e.g. `127.0.0.1:7856`
- `SALT`: the initial salt to obfuscate uri, e.g. `a secret string here`
- `URI_LEN`: the output short url length when shorting url, e.g. `12`
- `BUFFER_LEN`: the cached uri in memory, e.g. `1000`

# Speed & Optimization 

- Cached the most 1000-visited uris in memory
- Persistent the data into local sqlite database 
- More to go

# BenchMark

Generally, the result of benchmarking uror is various from hardware, Operating System, and depolyment environment and so on,

Tested On 4-thread HHD local server with [`wrk`](https://github.com/wg/wrk):
```bash 
$ wrk -t4 -c600 -d10s http://127.0.0.1:7856/\?uri=https://google.co.uk

Running 10s test @ http://127.0.0.1:7856/?uri=https://google.co.uk
  4 threads and 600 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.72ms    2.08ms  23.09ms   44.52%
    Req/Sec    40.50k     4.67k   61.07k    59.39%
  1299262 requests in 10.09s, 137.78MB read
Requests/sec:  128767.23
Transfer/sec:      13.65MB
```

Actually the performance expected to be better for server with better HardWare.
