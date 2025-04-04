//-- ./tests/api/helpers/spawn/client.rs

// #![allow(unused)] // For beginning only.

/// Spawn a Tonic Client for testing server endpoints
///
/// #### Reference
///
/// * [Tonic LND client](https://github.com/Kixunil/tonic_lnd/blob/master/src/lib.rs)
/// ---

/// This is part of public interface, so it's re-exported.
pub extern crate tonic;

use std::time;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

/// Convenience type alias for authentication client.
// pub type AuthenticationClient =
// authentication_service::rpc::proto::authentication_service_client::AuthenticationServiceClient<
//     InterceptedService<Channel, TokenInterceptor>>;
pub type AuthenticationClient =
authentication_service::rpc::proto::authentication_service_client::AuthenticationServiceClient<Channel>;

/// Convenience type alias for sessions client
pub type SessionsClient =
authentication_service::rpc::proto::sessions_service_client::SessionsServiceClient<
    InterceptedService<Channel, TokenInterceptor>>;

// Convenience type alias for users client
pub type UsersClient =
    authentication_service::rpc::proto::users_service_client::UsersServiceClient<
        InterceptedService<Channel, TokenInterceptor>>;

/// Tonic Client
#[derive(Clone)]
pub struct TonicClient {
    authentication: AuthenticationClient,
    sessions: SessionsClient,
    users: UsersClient,
}

impl TonicClient {
    /// Returns the authentication client.
    pub fn authentication(&mut self) -> &mut AuthenticationClient {
        &mut self.authentication
    }

    /// Returns the sessions client.
    pub fn sessions(&mut self) -> &mut SessionsClient {
        &mut self.sessions
    }

    /// Returns the users client.
    pub fn users(&mut self) -> &mut UsersClient {
        &mut self.users
    }

    /// Spawn a new tonic client based on the tonic server
    pub async fn spawn_client(
        server: &super::TonicServer,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Build Tonic Client channel
        let uri: tonic::transport::Uri = server.address.parse()?;
        let endpoint = Channel::builder(uri);
        let inner: Channel = endpoint.connect().await?;

        // Get tokens
        let access_token = server.clone().access_token;
        let refresh_token = server.clone().refresh_token;

        // Get access token duration
        let at_duration = time::Duration::new(
            (server.config.application.access_token_duration_minutes * 60)
                .try_into()
                .unwrap(),
            0,
        );        
        let at_duration =
            cookie::time::Duration::new(at_duration.as_secs() as i64, 0);

        // Get refresh token duration
        let rt_duration = time::Duration::new(
            (server.config.application.refresh_token_duration_minutes * 60)
                .try_into()
                .unwrap(),
            0,
        );

        // Build access cookie string
        let access_cookie =
            Cookie::build(("access_token", access_token.to_string()))
                // Set the domain of the cookie
                .domain(server.address.to_owned())
                // Indicates the path that must exist in the requested URL for the browser to send the Cookie header.
                .path("/")
                // Indicates the number of seconds until the cookie expires.
                .max_age(at_duration)
                // Forbids JavaScript from accessing the cookie
                .http_only(true)
                // Indicates that the cookie is sent to the server only when a request is made with the https or localhost
                .secure(false)
                .build()
                .to_string();

        // Build refresh token as a string
        let refresh_cookie = refresh_token
            .build_cookie(&server.address, &rt_duration)
            .to_string();

        // Create client token interceptor
        let client_interceptor = TokenInterceptor {
            access_cookie,
            refresh_cookie,
        };

        // Build Authentication client request
        let authentication = AuthenticationClient::new(inner.clone());

        // Build sessions client request
        let sessions = authentication_service::rpc::proto::sessions_service_client::SessionsServiceClient::with_interceptor(inner.clone(), client_interceptor.clone());

        // Build Users client request
        let users = authentication_service::rpc::proto::users_service_client::UsersServiceClient::with_interceptor(inner.clone(), client_interceptor.clone());

        let client = TonicClient {
            authentication,
            sessions,
            users,
        };

        Ok(client)
    }
}

/// Supplies requests with access token
#[derive(Clone)]
pub struct TokenInterceptor {
    access_cookie: String,
    refresh_cookie: String,
}

use cookie::Cookie;
use http::header::COOKIE;
use http::HeaderMap;
use tonic::metadata::MetadataMap;

impl tonic::service::Interceptor for TokenInterceptor {
    #[tracing::instrument(name = "Token Interceptor: ", skip_all)]
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // Create a new http header map
        let mut http_header = HeaderMap::new();
        // println!("Access Token: {:?}", self.access_cookie);
        // println!("Refresh Token: {:?}", self.refresh_cookie);

        // Add refresh cookie to the http header map
        http_header.append(COOKIE, self.access_cookie.parse().unwrap());
        http_header.append(COOKIE, self.refresh_cookie.parse().unwrap());

        // Add the http header to the rpc response
        *request.metadata_mut() = MetadataMap::from_headers(http_header);
        tracing::debug!("Added cookie headers to request: {:?}", request);

        Ok(request)
    }
}
