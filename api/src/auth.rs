use std::{
    env,
    lazy::{Lazy, OnceCell, SyncOnceCell},
    sync::{Once, RwLock},
    time::Duration,
};

use futures::prelude::*;
use jsonwebtoken::jwk;
use miette::Diagnostic;
use poem::{error::ResponseError, http::StatusCode, FromRequest, Request, RequestBody};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::IntervalStream;
use tracing::Instrument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub aud: Vec<String>,
    pub sub: String,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for Claims {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> poem::Result<Self> {
        let token = req
            .headers()
            .get("Authorization")
            .ok_or_else(|| AuthError::MissingAuthorizationHeader)?
            .to_str()
            .map_err(|_| AuthError::FailedStringifyingAuthorizationHeader)?
            .strip_prefix("Bearer ")
            .ok_or(AuthError::AuthorizationHeaderMissingBearer)?;

        let jwks = JWKS.get().ok_or(AuthError::MissingJwks)?.read().unwrap();

        let header = jsonwebtoken::decode_header(token).map_err(AuthError::JsonWebToken)?;
        let kid = header.kid.ok_or(AuthError::MissingKidClaim)?;
        let key = jwks.find(&kid).ok_or(AuthError::MissingKey)?;

        match &key.algorithm {
            jwk::AlgorithmParameters::RSA(rsa) => jsonwebtoken::decode::<Claims>(
                &token,
                &jsonwebtoken::DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                    .map_err(AuthError::JsonWebToken)?,
                &jsonwebtoken::Validation::new(
                    key.common
                        .algorithm
                        .unwrap_or(jsonwebtoken::Algorithm::RS256),
                ),
            )
            .map(|data| data.claims)
            .map_err(AuthError::JsonWebToken)
            .map_err(Into::into),
            algorithm @ _ => Err(AuthError::UnsupportedAlgorithm(algorithm.clone()).into()),
        }
    }
}

static JWKS: SyncOnceCell<RwLock<jwk::JwkSet>> = SyncOnceCell::new();

pub fn update_jwks() -> impl Stream<Item = Result<(), AuthError>> {
    IntervalStream::new(tokio::time::interval(Duration::from_secs(5 * 60))).then(|_| {
        async move {
            let jwks_url = env::var("OIL_JWKS_URL").map_err(AuthError::MissingJwksUrl)?;

            let jwks = reqwest::get(&jwks_url)
                .await
                .map_err(AuthError::FailedFetchingJwks)?
                .json::<jwk::JwkSet>()
                .await
                .map_err(AuthError::FailedFetchingJwks)?;

            tracing::info!(?jwks, "updated jwks");

            let rw_lock = JWKS.get_or_init(|| RwLock::new(jwks.clone()));
            *rw_lock.write().unwrap() = jwks;

            Ok::<_, AuthError>(())
        }
        .instrument(tracing::info_span!("update_jwks"))
    })
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum AuthError {
    #[error("missing `Authorization` header")]
    MissingAuthorizationHeader,

    #[error("invalid characters were present in the `Authorization` header")]
    FailedStringifyingAuthorizationHeader,

    #[error("malformed `Authorization` header: missing `Bearer ` prefix")]
    AuthorizationHeaderMissingBearer,

    #[error("json web token error")]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),

    #[error("missing JWK set")]
    MissingJwks,

    #[error("missing `kid` claim necessary for JWT verification")]
    MissingKidClaim,

    #[error("missing key corresponding to `kid` claim")]
    #[diagnostic(help("do we need to refresh the keys?"))]
    MissingKey,

    #[error("unsupported algorithm: {0:#?}")]
    #[diagnostic(url("https://auth0.com/docs/get-started/applications/signing-algorithms"))]
    UnsupportedAlgorithm(jwk::AlgorithmParameters),

    #[error("JWK URL not set")]
    #[diagnostic(help("set `OIL_JWKS_URL` to the URL of your JWK set"))]
    MissingJwksUrl(#[source] env::VarError),

    #[error("failed to fetch JWKs")]
    FailedFetchingJwks(#[source] reqwest::Error),
}

impl ResponseError for AuthError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            Self::MissingAuthorizationHeader => StatusCode::UNAUTHORIZED,
            Self::FailedStringifyingAuthorizationHeader => StatusCode::BAD_REQUEST,
            Self::AuthorizationHeaderMissingBearer => StatusCode::BAD_REQUEST,
            Self::JsonWebToken(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingKidClaim => StatusCode::BAD_REQUEST,
            Self::MissingJwks => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingKey => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnsupportedAlgorithm(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingJwksUrl(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FailedFetchingJwks(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> poem::Response
    where
        Self: std::error::Error + Send + Sync + 'static,
    {
        use miette::{GraphicalReportHandler, GraphicalTheme, ReportHandler};
        use std::fmt::{self, Display};

        struct PrettyDiagnostic<'d, D>(&'d D);

        impl<'d, D: Diagnostic> Display for PrettyDiagnostic<'d, D> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                GraphicalReportHandler::new()
                    .with_theme(GraphicalTheme::none())
                    .debug(self.0, f)
            }
        }

        poem::Response::builder()
            .status(self.status())
            .body(format!("{}", PrettyDiagnostic(self)))
    }
}
