## Rust Remote Storage Adapter for Prometheus Remote Write

This is a write adapter that receives samples via the Prometheus remote write
protocol and stores them in Graphite (InfluxDB, OpenTSDB not added yet). This is
the Rust equivalent to the [official Prometheus remote storage adapter implementation](https://github.com/prometheus/prometheus/tree/main/documentation/examples/remote_storage/remote_storage_adapter)

## Example: Graphite

Start Graphite with

``` 
docker run -d \
 --name graphite \    
 --restart=always \
 -p 80:80 \
 -p 2003-2004:2003-2004 \
 -p 2023-2024:2023-2024 \
 -p 8125:8125/udp \
 -p 8126:8126 \
 graphiteapp/graphite-statsd
 ```

Run

```
export INGEST_PORT=3030
```

and configure a Prometheus remote writer to send metrics to `$INGEST_PORT/write`, e.g.:

```
remote_write:
  - url: "http://localhost:3030/write/"
```

Now run the binary with

```
remote_storage_adapter --ingest-port=$INGEST_PORT --graphite-address=localhost:2003
```


 The Graphite Web UI is accessible at `localhost` (default username/password is `admin:admin`).
