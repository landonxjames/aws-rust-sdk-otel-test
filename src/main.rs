use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{config::Region, meta::PKG_VERSION, Client, Error};
use once_cell::sync::Lazy;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![opentelemetry::KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "lnj-test-service",
    )])
});

#[tokio::main]
async fn main() -> Result<(), Error> {
    // env_logger::init();
    // tracing_subscriber::fmt::init();

    let tracing_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default().with_resource(RESOURCE.clone()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracer = tracing_provider.tracer("TEST_APPLICATION");
    global::set_tracer_provider(tracing_provider);

    let logging_provider = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_resource(RESOURCE.clone())
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    // let fmt_layer = tracing_subscriber::fmt::layer();
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let trace_appender_layer = OpenTelemetryTracingBridge::new(&logging_provider);

    tracing_subscriber::registry()
        // .with(fmt_layer)
        .with(telemetry_layer)
        // .with(trace_appender_layer)
        .init();

    // Uncommenting these lines calling a function annotated with
    // #[tracing::instrument()] and running them with the AWS specific
    // code below commented out leads to no logs being pushed to Jaeger.
    // When these are present with the AWS code they will show up in Jaeger
    // only when the "OpenTelemetry trace error occurred." log appears.
    // do_the_thing(1);
    // do_the_thing(2);
    // do_the_thing(3);
    // do_the_thing(4);
    // do_the_thing(5);

    // Comment out below block to remove AWS bits from this example
    let region_provider =
        RegionProviderChain::first_try(Region::new("us-west-2")).or_default_provider();
    println!();
    println!("DynamoDB client version: {}", PKG_VERSION);
    println!(
        "Region:                  {}",
        region_provider.region().await.unwrap().as_ref()
    );
    println!();
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&shared_config);
    let resp = client.list_tables().send().await?;
    println!("Tables:");
    let names = resp.table_names();
    let len = names.len();
    for name in names {
        println!("  {}", name);
    }
    println!("Found {} tables", len);

    global::shutdown_tracer_provider();
    Ok(())
}

#[tracing::instrument()]
fn do_the_thing(num: u8) {
    tracing::info!("I AM DOING THE THING {num}");
}
