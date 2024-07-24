use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{config::Region, meta::PKG_VERSION, Client, Error};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // env_logger::init();
    // tracing_subscriber::fmt::init();

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "my-test-service",
            )]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracer = provider.tracer("TEST_APPLICATION");

    // let fmt_layer = tracing_subscriber::fmt::layer();
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        // .with(fmt_layer)
        .with(telemetry_layer)
        .init();

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
    do_the_thing();
    let _span = tracing::debug_span!("MY DEBUG SPAN").entered();

    Ok(())
}

#[tracing::instrument()]
fn do_the_thing() {
    tracing::info!("I AM DOING THE THING");
}
