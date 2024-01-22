use std::collections::HashSet;

use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    password: String,

    created_at: chrono::DateTime<chrono::Utc>,
    pub verified: bool,
    verification_code: Option<String>,
    code_expires_at: Option<chrono::DateTime<chrono::Utc>>,

    pub otp_enabled: bool,
    pub otp_verified: bool,
    pub otp_secret: Option<String>,
    otp_auth_url: Option<String>,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password", &"[redacted]")
            .field("created_at", &self.created_at)
            .field("verified", &self.verified)
            .field("verification_code", &"[redacted]")
            .field("code_expires_at", &self.code_expires_at)
            .field("otp_enabled", &self.otp_enabled)
            .field("otp_verified", &self.otp_verified)
            .field("otp_secret", &"[redacted]")
            .field("otp_auth_url", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes() // We use the password hash as the auth
                                 // hash--what this means
                                 // is when the user changes their password the
                                 // auth session becomes invalid.
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct SignUpCredentials {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone)]
pub struct Backend {
    db: PgPool,
}

impl Backend {
    pub const fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = LoginCredentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as!(
            Self::User,
            r#"
            select id, username, email, password, created_at, verified, verification_code, code_expires_at, otp_enabled, otp_verified, otp_secret, otp_auth_url
            from users
            where username = $1
            "#,
            creds.username
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user.filter(|user| verify_password(creds.password.as_bytes(), &user.password).is_ok()))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            select id, username, email, password, created_at, verified, verification_code, code_expires_at, otp_enabled, otp_verified, otp_secret, otp_auth_url
            from users
            where id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, FromRow)]
pub struct Permission {
    pub name: String,
}

impl From<&str> for Permission {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl AuthzBackend for Backend {
    type Permission = Permission;

    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let permissions: Vec<Self::Permission> = sqlx::query_as!(
            Permission,
            r#"
            select distinct permissions.name
            from users
            join users_groups on users.id = users_groups.user_id
            join groups_permissions on users_groups.group_id = groups_permissions.group_id
            join permissions on groups_permissions.permission_id = permissions.id
            where users.id = $1
            "#,
            user.id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(permissions.into_iter().collect())
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
