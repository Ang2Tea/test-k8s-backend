use anyhow::{Context, Result};
use opentelemetry::{KeyValue, global, trace::TracerProvider as _};
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::attribute::{
    DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION,
};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Ошибка при остановке трейсера: {err:?}");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Ошибка при остановке метрик: {err:?}");
        }
    }
}

fn resource() -> Result<Resource> {
    let environment = std::env::var("APP_ENV").context("APP_ENV not found")?;

    Ok(Resource::builder()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, environment),
        ])
        .build())
}

fn init_meter_provider() -> Result<SdkMeterProvider> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .context("OTEL_EXPORTER_OTLP_ENDPOINT not found")?;

    let interval_secs = std::env::var("OTEL_METRICS_INTERVAL_SECS")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()
        .context("Не удалось распарсить OTEL_METRICS_INTERVAL_SECS в число")?;

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .context("Не удалось создать MetricExporter для OTLP")?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(interval_secs))
        .build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource()?)
        .with_reader(reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

fn init_tracer_provider() -> Result<SdkTracerProvider> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .context("OTEL_EXPORTER_OTLP_ENDPOINT not found")?;

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .context("Не удалось создать SpanExporter для OTLP")?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource()?)
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(tracer_provider.clone());
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    Ok(tracer_provider)
}

pub fn init() -> Result<OtelGuard> {
    // Получаем провайдеры или пробрасываем ошибку выше
    let meter_provider = init_meter_provider().context("Ошибка инициализации провайдера метрик")?;
    let tracer_provider =
        init_tracer_provider().context("Ошибка инициализации провайдера трейсов")?;

    let tracer = tracer_provider.tracer(env!("CARGO_PKG_NAME"));

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer().json().with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(MetricsLayer::new(meter_provider.clone()))
        .with(OpenTelemetryLayer::new(tracer))
        .try_init() // Используем try_init() вместо init(), чтобы вернуть ошибку в случае конфликта регистраторов
        .context("Не удалось инициализировать глобальный tracing_subscriber")?;

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}
