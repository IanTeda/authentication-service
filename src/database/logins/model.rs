//-- ./src/database/logins/model.rs

// #![allow(unused)] // For development only

//! The Logins database model
//! ---

use std::net::Ipv4Addr;

use chrono::{DateTime, SubsecRound, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, sqlx::FromRow, serde::Deserialize)]
pub struct Logins {
    pub id: Uuid,
    pub user_id: Uuid,
    pub login_on: DateTime<Utc>,
    pub login_ip: Option<i32>,
}

impl Logins {
    // pub fn login_ip(&self) -> Option<Ipv4Addr> {
    //     match self.login_ip {
    //         Some(x) => Some(self.login_ip.into()),
    //         None => None,
    //     }
    // }
    pub fn new(user_id: &Uuid, login_ip: Option<Ipv4Addr>) -> Self {
        let id = Uuid::now_v7();
        let user_id = user_id.to_owned();
        let login_on = Utc::now().round_subsecs(0);
        let login_ip = match login_ip {
            Some(ip_address) => {
                let ip_address = u32::from(ip_address.to_owned()) as i32;
                Some(ip_address)
            }
            None => None,
        };

        Logins {
            id,
            user_id,
            login_on,
            login_ip,
        }
    }

    #[cfg(test)]
    pub fn mock_data(user_id: &Uuid) -> Result<Self, crate::prelude::BackendError> {
        use crate::utils;
        use chrono::SubsecRound;
        use fake::faker::chrono::en::DateTime;
        use fake::faker::internet::en::IPv4;
        use fake::Fake;

        // Generate random Uuid V7
        let random_id = utils::mock_uuid();
        // Generate random DateTime
        let random_login_on: DateTime<Utc> = DateTime().fake();
        // Adjust order of accuracy to be consistent with Postgres
        let random_login_on = random_login_on.round_subsecs(0);
        // Generate random IPV4 address
        let random_ip: Ipv4Addr = IPv4().fake();
        // Convert IPV4 to an i32 to be consistent with Postgres INT type
        let random_ip = u32::from(random_ip) as i32;

        Ok(Logins {
            id: random_id,
            user_id: user_id.to_owned(),
            login_on: random_login_on,
            login_ip: Some(random_ip),
        })
    }
}

//-- Unit Tests
#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::IPv4, Fake};
    use sqlx::{Pool, Postgres};
    use tracing_subscriber::registry::Data;

    use crate::database::{self, Users};

    use super::*;

    // Override with more flexible result and error
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    // Test inserting into database
    #[test]
    fn create_new_login() -> Result<()> {
        //-- Setup and Fixtures (Arrange)
        let random_user = database::Users::mock_data()?;
        let random_login_ip: Ipv4Addr = IPv4().fake();
        let random_login_ip = Some(random_login_ip);
        
        //-- Execute Function (Act)
        let database_record = database::Logins::new(&random_user.id, random_login_ip);

        //-- Checks (Assertions)
        assert_eq!(database_record.user_id, random_user.id);
        assert_eq!(database_record.login_ip.unwrap(), u32::from(random_login_ip.unwrap()) as i32);

        //-- Return
        Ok(())
    }
}
