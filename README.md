A minimal example of using otel telemetry with the AWS Rust SDK. To run first start an instance of [Jaeger](https://www.jaegertracing.io/):

```
docker run -d --name jaeger -e COLLECTOR_OTLP_ENABLED=true -p 16686:16686 -p 4317:4317 -p 4318:4318 jaegertracing/all-in-one:latest
```

Then simply run the program:

```
cargo run --release
```

The output should look something like:

```
DynamoDB client version: 1.38.0
Region:                  us-west-2

Tables:
  MyTable1
  MyTable2
Found 2 tables
OpenTelemetry trace error occurred. cannot send message to batch processor as the channel is closed
OpenTelemetry trace error occurred. cannot send message to batch processor as the channel is closed
```

Note that the seeming error lines at the end are not always present. Strangly when they are present is when the metrics actually show up in the Jaeger instance. When the error logs are not present the logs do not appear in Jaeger. The error is present (and thus the logs show up) ~25% of the time.
