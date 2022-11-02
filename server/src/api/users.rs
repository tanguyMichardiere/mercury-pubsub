use axum::extract::{Path, State};
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use self::error::{Error, Result};
use crate::api::extract::validated_json::ValidatedJson;
use crate::models::user::User;
use crate::state::SharedState;

pub(crate) fn app(state: SharedState) -> Router<SharedState> {
    Router::with_state(state)
        .route("/", get(list_users).post(create_user))
        .route("/rename", patch(rename))
        .route("/change-password", patch(change_password))
        .route("/:id", delete(delete_user))
}

/// Get self and all lower ranked users.
#[instrument]
async fn list_users(State(state): State<SharedState>, user: User) -> Result<Json<Vec<User>>> {
    let min_rank = user.rank + 1;
    let mut users = vec![user];
    users.append(&mut User::get_all(&state.read().await.pool, min_rank).await?);
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
    State(state): State<SharedState>,
    user: User,
    ValidatedJson(body): ValidatedJson<CreateUserBody>,
) -> Result<(StatusCode, Json<User>)> {
    Ok((
        StatusCode::CREATED,
        Json(
            User::new(
                &state.read().await.pool,
                &body.name,
                &body.password,
                user.rank + 1,
            )
            .await?,
        ),
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
    State(state): State<SharedState>,
    mut user: User,
    ValidatedJson(body): ValidatedJson<RenameBody>,
) -> Result<Json<User>> {
    user.rename(&state.read().await.pool, &body.name).await?;
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
    State(state): State<SharedState>,
    mut user: User,
    ValidatedJson(body): ValidatedJson<ChangePasswordBody>,
) -> Result<Json<User>> {
    user.change_password(&state.read().await.pool, &body.password)
        .await?;
    Ok(Json(user))
}

/// Delete a user.
#[instrument]
async fn delete_user(
    State(state): State<SharedState>,
    user: User,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let other_user = User::get(&state.read().await.pool, id).await?;
    if other_user.rank > user.rank {
        other_user.delete(&state.read().await.pool).await?;
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
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            debug!(?self);
            match self {
                Error::UserError(error) => error.into_response(),
                Error::HigherRankUser => {
                    (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
                }
            }
        }
    }
}
