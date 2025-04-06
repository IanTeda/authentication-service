//-- ./src/services/mods.rs

/// # Services Module
/// 
/// This module contains all the services used in the application.
/// It is the main entry point for all service-related functionality.
///
/// ## Services
/// - **AuthenticationService**: Handles user authentication and authorization.
/// - **SessionsService**: Manages user sessions and session-related data.
/// - **UsersService**: Manages user data and user-related operations.
/// - **UtilitiesService**: Provides utility functions and helpers.
///
/// ## References
/// - [govinda-attal/app-a](https://github.com/govinda-attal/app-a/tree/2-grpc-server)

// #![allow(unused)] // For beginning only.

// Flatten module exports
pub use authentication::AuthenticationService;
pub use sessions::SessionsService;
pub use users::UsersService;
pub use utilities::UtilitiesService;

mod authentication;
mod sessions;
mod users;
mod utilities;
