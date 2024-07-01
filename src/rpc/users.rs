//-- ./src/rpc/users.rs

//! Return a result containing a RPC Utilities server

#![allow(unused)] // For development only

use crate::prelude::*;

// use super::proto::users_server::Users;
// use super::proto::{CreateUserRequest, UserResponse};

use tonic::{Request, Response, Status};