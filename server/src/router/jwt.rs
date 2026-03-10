use crate::error::*;
use crate::service::UserService;
use crate::AppState;
use axum::body::Body;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use axum::{http, Json, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// 生成Jwt后最大有效时间，按分钟
const JWT_MAX_EXPIRATION: usize = 60 * 24 * 30 * 6;

pub async fn authorize(
    state: State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(NihilityServerError::MissingCredentials);
    }
    if !UserService::check_user_password(&state.conn, payload.username.clone(), payload.password)
        .await?
    {
        return Err(NihilityServerError::WrongCredentials);
    }

    let claims = Claims {
        sub: payload.username,
        exp: calculate_expiry(state.jwt.expiration)?,
    };
    let token = encode(&Header::default(), &claims, &state.jwt.encoding)?;

    Ok(Json(AuthBody::new(token)))
}

pub(crate) async fn auth_middleware(
    State(keys): State<JwtKeys>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response> {
    let bearer = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(NihilityServerError::InvalidToken)?
        .strip_prefix("Bearer ")
        .ok_or(NihilityServerError::InvalidToken)?;

    let token_data = decode::<Claims>(bearer, &keys.decoding, &Validation::default())
        .map_err(|_| NihilityServerError::InvalidToken)?;

    request
        .headers_mut()
        .insert("x-username", HeaderValue::from_str(&token_data.claims.sub)?);

    Ok(next.run(request).await)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    username: String,
    password: String,
}

#[derive(Clone)]
pub(crate) struct JwtKeys {
    /// 过期时间（分钟）
    expiration: usize,
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl<S> FromRequestParts<S> for Claims
where
    JwtKeys: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = NihilityServerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> core::result::Result<Self, Self::Rejection> {
        let state = JwtKeys::from_ref(state);
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| NihilityServerError::InvalidToken)?;
        let token_data = decode::<Claims>(bearer.token(), &state.decoding, &Validation::default())
            .map_err(|_| NihilityServerError::InvalidToken)?;
        Ok(token_data.claims)
    }
}

impl JwtKeys {
    pub(crate) fn new(secret: &[u8], expiration: usize) -> Self {
        Self {
            expiration,
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

pub fn calculate_expiry(minutes_from_now: usize) -> Result<usize> {
    if minutes_from_now > JWT_MAX_EXPIRATION {
        return Err(NihilityServerError::Config(format!(
            "Jwt expiration too high: {}",
            minutes_from_now
        )));
    }

    let now = Utc::now();
    let expiry = now
        .checked_add_signed(Duration::minutes(minutes_from_now.try_into().map_err(
            |_| {
                NihilityServerError::Config(format!(
                    "Jwt expiration Too long: {}",
                    minutes_from_now
                ))
            },
        )?))
        .ok_or(NihilityServerError::Config(format!(
            "Jwt expiration Too long: {}",
            minutes_from_now
        )))?;

    usize::try_from(expiry.timestamp()).map_err(|_| {
        NihilityServerError::Config(format!("Jwt expiration Too long: {}", expiry.timestamp()))
    })
}
