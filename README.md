Simple http web server, mostly based on rotor-http.

* Always returns the same response.
* Does not parse http headers.
* Keeps connection alive (for benchmarking).
* Intended for comparison with UV version: https://github.com/jmquigs/libuv-webserver
