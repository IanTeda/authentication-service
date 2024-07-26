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

use tonic::codegen::InterceptedService;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

/// Convenience type alias for authentication client.
pub type AuthenticationClient =
authentication_microservice::rpc::proto::authentication_client::AuthenticationClient<Channel>;

/// Convenience type alias for refresh token client
pub type RefreshTokensClient =
authentication_microservice::rpc::proto::refresh_tokens_client::RefreshTokensClient<
    InterceptedService<Channel, AccessTokenInterceptor>,
>;

// Convenience type alias for users client
pub type UsersClient =
    authentication_microservice::rpc::proto::users_client::UsersClient<
        InterceptedService<Channel, AccessTokenInterceptor>,
    >;

pub type LoginsClient =
    authentication_microservice::rpc::proto::logins_client::LoginsClient<
        InterceptedService<Channel, AccessTokenInterceptor>,
    >;

/// Tonic Client
#[derive(Clone)]
pub struct TonicClient {
    authentication: AuthenticationClient,
    refresh_tokens: RefreshTokensClient,
    users: UsersClient,
    logins: LoginsClient,
}

impl TonicClient {
    /// Returns the authentication client.
    pub fn authentication(&mut self) -> &mut AuthenticationClient {
        &mut self.authentication
    }

    /// Returns the refresh tokens client.
    pub fn refresh_tokens(&mut self) -> &mut RefreshTokensClient {
        &mut self.refresh_tokens
    }

    /// Returns the users client.
    pub fn users(&mut self) -> &mut UsersClient {
        &mut self.users
    }

    /// Returns the logins client.
    pub fn logins(&mut self) -> &mut LoginsClient {
        &mut self.logins
    }

    //noinspection RsUnnecessaryQualifications
    //noinspection RsUnnecessaryQualifications
    /// Spawn a new tonic client based on the tonic server
    pub async fn spawn_client(
        server: &super::TonicServer,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Build Tonic Client channel
        let uri: tonic::transport::Uri = server.address.parse()?;
        let endpoint = Channel::builder(uri);
        let inner: Channel = endpoint.connect().await?;

        // Build client interceptor that adds the access-token to the request
        let access_token = server.clone().access_token;
        let interceptor = AccessTokenInterceptor { access_token };

        // Build Authentication client request
        let authentication = AuthenticationClient::new(inner.clone());

        // Build Refresh Tokens client request
        let refresh_tokens = authentication_microservice::rpc::proto::refresh_tokens_client::RefreshTokensClient::with_interceptor(inner.clone(), interceptor.clone());

        // Build Users client request
        let users = authentication_microservice::rpc::proto::users_client::UsersClient::with_interceptor(inner.clone(), interceptor.clone());

        let logins = authentication_microservice::rpc::proto::logins_client::LoginsClient::with_interceptor(inner.clone(), interceptor.clone());

        let client = TonicClient {
            authentication,
            refresh_tokens,
            users,
            logins,
        };

        Ok(client)
    }
}

/// Supplies requests with access token
#[derive(Clone)]
pub struct AccessTokenInterceptor {
    access_token: String,
}

impl tonic::service::Interceptor for AccessTokenInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // let token: tonic::metadata::MetadataValue<_> = "Bearer some-auth-token".parse().unwrap();
        let token: MetadataValue<_> = self.access_token.parse().unwrap();
        // println!("access_token: {token:#?}");

        request.metadata_mut().append("access_token", token.clone());
        // println!("Access token appended");

        Ok(request)
    }
}
