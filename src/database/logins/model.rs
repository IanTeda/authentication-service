//-- ./src/database/logins/model.rs

// #![allow(unused)] // For development only

//! The Logins database model
//! ---

use std::net::Ipv4Addr;

use chrono::{DateTime, Utc};
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
