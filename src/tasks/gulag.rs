use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::model::id::{RoleId, UserId};
use std::fmt::{self, Display, Result as FmtResult};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Gulag {
    pub user: (String, UserId),
    pub roles: Vec<(String, RoleId)>,
    pub end: DateTime<Utc>,
}

impl Gulag {
    pub fn time_to_act(&self) -> bool {
        self.end >= Utc::now()
    }
}

impl Display for Gulag {
    fn fmt(&self, f: &mut fmt::Formatter) -> FmtResult {
        writeln!(
            f,
            "Gulag entry for: '{}' (ID: {})",
            self.user.0, self.user.1
        )?;
        writeln!(f, "    Roles to restore:")?;
        for (role_name, role_id) in &self.roles {
            writeln!(f, "        - '{}' (ID: {})", role_name, role_id)?;
        }
        writeln!(f, "    End of sentence: {}", self.end)?;
        Ok(())
    }
}
