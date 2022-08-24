use axum::extract::Path;
use axum::routing::{delete, get, patch};
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::api::extract::validated_json::ValidatedJson;
use crate::models::user::User;

use error::{Error, Result};

pub(crate) fn app() -> Router {
    Router::new()
        .route("/", get(list_users).post(create_user).delete(delete_self))
        .route("/rename", patch(rename))
        .route("/change-password", patch(change_password))
        .route("/:id", delete(delete_user))
}

/// Get self and all lower ranked users.
#[instrument]
async fn list_users(Extension(pool): Extension<PgPool>, user: User) -> Result<Json<Vec<User>>> {
    let min_rank = user.rank + 1;
    let mut users = vec![user];
    users.append(&mut User::get_all(&pool, min_rank).await?);
    Ok(Json(users))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreateUserBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
    #[validate(length(min = 8))]
    password: String,
}

/// Create a user.
///
/// The new user's rank will be equal to its parent user's rank + 1.
#[instrument]
async fn create_user(
    Extension(pool): Extension<PgPool>,
    user: User,
    ValidatedJson(body): ValidatedJson<CreateUserBody>,
) -> Result<(StatusCode, Json<User>)> {
    Ok((
        StatusCode::CREATED,
        Json(User::new(&pool, &body.name, &body.password, user.rank + 1).await?),
    ))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct RenameBody {
    #[validate(length(min = 4, max = 16))]
    name: String,
}

/// Rename self.
#[instrument]
async fn rename(
    Extension(pool): Extension<PgPool>,
    mut user: User,
    ValidatedJson(body): ValidatedJson<RenameBody>,
) -> Result<Json<User>> {
    user.rename(&pool, &body.name).await?;
    Ok(Json(user))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct ChangePasswordBody {
    #[validate(length(min = 8))]
    password: String,
}

/// Change own password.
#[instrument]
async fn change_password(
    Extension(pool): Extension<PgPool>,
    mut user: User,
    ValidatedJson(body): ValidatedJson<ChangePasswordBody>,
) -> Result<Json<User>> {
    user.change_password(&pool, &body.password).await?;
    Ok(Json(user))
}

/// Delete self.
#[instrument]
async fn delete_self(Extension(pool): Extension<PgPool>, user: User) -> Result<StatusCode> {
    if user.rank > 0 {
        user.delete(&pool).await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(Error::DeleteRootUser)
    }
}

/// Delete a user.
#[instrument]
async fn delete_user(
    Extension(pool): Extension<PgPool>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let other_user = User::get(&pool, id).await?;
    if other_user.rank > user.rank {
        other_user.delete(&pool).await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(Error::HigherRankUser)
    }
}

mod error {
    use axum::response::IntoResponse;
    use hyper::StatusCode;
    use tracing::debug;

    use crate::models::user;

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum Error {
        #[error(transparent)]
        UserError(#[from] user::error::Error),
        #[error("Cannot delete a higher ranked user")]
        HigherRankUser,
        #[error("Cannot delete the root user")]
        DeleteRootUser,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Error::UserError(error) => error.into_response(),
                Error::HigherRankUser => {
                    (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
                }
                Error::DeleteRootUser => {
                    (StatusCode::BAD_REQUEST, self.to_string()).into_response()
                }
            }
        }
    }
}
