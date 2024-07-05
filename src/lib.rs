//-- ./src/lib.rs

#![doc = include_str!("../README.md")]

pub mod configuration;
pub mod database;
pub mod domains;
pub mod error;
pub mod prelude;
pub mod middleware;
pub mod reflections;
pub mod router;
pub mod rpc;
pub mod services;
pub mod startup;
pub mod telemetry;
pub mod utilities;
