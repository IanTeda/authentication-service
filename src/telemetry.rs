// -- ./src/telemetry.rs

// #![allow(unused)] // For beginning only.

//! Sets up the API log telemetry
//!
//! # Application Telemetry
//!
//! Instrumenting to collect structured, event-based diagnostic information.
//!
//! Tracing is made up of spans, events within those spans and subscribers that
//! pick what spans and events to grab and and what then performs tasks on the
//! grabbed spans and events.
//!
//! # References
//!
//! Learn more about Rust Telemetry (i.e async logging)
//!
//! * [Tracing Crate Documentation](https://docs.rs/tracing/latest/tracing/)
//! * [Tracing Repo](https://github.com/tokio-rs/tracing)
//! * [Decrusting the tracing crate](https://youtu.be/21rtHinFA40)
//! * [Getting started with Tracing](https://tokio.rs/tokio/topics/tracing)
//! * [Can we have easier pretty log for development?](https://github.com/LukeMathWalker/tracing-bunyan-formatter/issues/17)

// TODO: Add https://prometheus.io/
// TODO: Add https://opentelemetry.io/
// TODO: Add tracing console

use crate::prelude::*;

use tracing::level_filters::LevelFilter;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{
    fmt::format::FmtSpan,
    layer::SubscriberExt,
    EnvFilter,
};

pub fn init() -> Result<(), BackendError>{
    //-- 1. Filter events
    // Default log level if info
    let default_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    // Try to use env runtime level, if not present use default
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| default_env_filter);

    // Build event collector for console output
    let console_collector = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    //-- 2. Build a registry of collectors
    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_collector);

    // Convert all log records into tracing events.
    let _log_tracer = tracing_log::LogTracer::init()?;

    //-- 3. Initiate tracing
    set_global_default(registry)?;

    Ok(())

}