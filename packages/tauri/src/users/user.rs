use serde::{Deserialize, Serialize};

use crate::git;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: String,
    pub picture: String,
    pub locale: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub access_token: String,
    pub github_access_token: Option<String>,
    #[serde(default)]
    pub github_username: Option<String>,
    #[serde(default)]
    pub current_project: Option<String>,
}

impl TryFrom<User> for git::Signature<'_> {
    type Error = git::Error;

    fn try_from(value: User) -> Result<Self, Self::Error> {
        git::Signature::now(&value.name, &value.email)
    }
}
