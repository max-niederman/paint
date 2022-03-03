use futures::prelude::*;
use jsonwebtoken::jwk;
use miette::Diagnostic;
use poem::{error::ResponseError, http::StatusCode, Request, RequestBody};
use poem_openapi::{ApiExtractor, ApiExtractorType};
use serde::{Deserialize, Serialize};
use tokio::time::{Interval, interval};
use tokio_stream::wrappers::IntervalStream;
use std::{env, lazy::SyncOnceCell, sync::RwLock, time::Duration};
use tracing::Instrument;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ClaimsSerialized")]
pub struct Claims {
    pub aud: Vec<String>,
    pub sub: String,
    pub scope: Vec<String>,
}

impl Claims {
    pub fn ensure_scopes<I, S>(&self, scopes: I) -> Result<(), AuthError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for scope in scopes {
            if !self.scope.iter().any(|s| s == scope.as_ref()) {
                return Err(AuthError::MissingScope {
                    missing: scope.as_ref().to_string(),
                    present: self.scope.clone(),
                });
            }
        }
        Ok(())
    }
}

#[poem::async_trait]
impl<'a> ApiExtractor<'a> for Claims {
    const TYPE: poem_openapi::ApiExtractorType = ApiExtractorType::SecurityScheme;

    const PARAM_IS_REQUIRED: bool = true;

    type ParamType = ();

    type ParamRawType = ();

    async fn from_request(
        request: &'a Request,
        _body: &mut RequestBody,
        _param_opts: poem_openapi::ExtractParamOptions<Self::ParamType>,
    ) -> poem::Result<Self> {
        let token = request
            .headers()
            .get("Authorization")
            .ok_or(AuthError::MissingAuthorizationHeader)?
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
                token,
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
            algorithm => Err(AuthError::UnsupportedAlgorithm(algorithm.clone()).into()),
        }
    }

    // we do this manually because we want to be able to return a custom error
    // otherwise we could just use the `poem_openapi::SecurityScheme` derive macro

    fn register(registry: &mut poem_openapi::registry::Registry) {
        registry.create_security_scheme(
            "Claims",
            poem_openapi::registry::MetaSecurityScheme {
                ty: "http",
                description: Some(
                    "JWT authentication/authorization via Auth0 using the Bearer scheme",
                ),
                name: None,
                key_in: None,
                scheme: Some("bearer"),
                bearer_format: None,
                flows: None,
                openid_connect_url: None,
            },
        )
    }

    fn security_scheme() -> Option<&'static str> {
        Some("Claims")
    }
}

static JWKS: SyncOnceCell<RwLock<jwk::JwkSet>> = SyncOnceCell::new();

pub fn update_jwks(period: Duration) -> impl Stream<Item = Result<(), AuthError>> {
    IntervalStream::new(interval(period)).then(|_| {
        async move {
            let jwks_url = env::var("OIL_JWKS_URL").map_err(AuthError::MissingJwksUrl)?;

            let jwks = reqwest::get(&jwks_url)
                .await
                .map_err(AuthError::FailedFetchingJwks)?
                .json::<jwk::JwkSet>()
                .await
                .map_err(AuthError::FailedFetchingJwks)?;

            tracing::info!(jwks_kids = ?jwks.keys.iter().map(|jwk| jwk.common.key_id.as_ref().unwrap()).collect::<Vec<_>>(), "updated jwks");

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

    #[error("missing requisite scope: \"{missing}\" not found in {present:#?}")]
    MissingScope {
        missing: String,
        present: Vec<String>,
    },
}

impl ResponseError for AuthError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            Self::MissingAuthorizationHeader => StatusCode::UNAUTHORIZED,
            Self::FailedStringifyingAuthorizationHeader => StatusCode::BAD_REQUEST,
            Self::AuthorizationHeaderMissingBearer => StatusCode::BAD_REQUEST,
            Self::JsonWebToken(err) => match err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                jsonwebtoken::errors::ErrorKind::InvalidAlgorithm => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                _ => StatusCode::UNAUTHORIZED,
            },
            Self::MissingKidClaim => StatusCode::BAD_REQUEST,
            Self::MissingJwks => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingKey => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnsupportedAlgorithm(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingJwksUrl(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FailedFetchingJwks(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingScope { .. } => StatusCode::UNAUTHORIZED,
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

// we deserialize the claims as [`ClaimsSerialized`] so that we can convert `scope` to a [`Vec<String>`]

#[derive(Deserialize)]
struct ClaimsSerialized {
    aud: Vec<String>,
    sub: String,
    scope: String,
}

impl From<ClaimsSerialized> for Claims {
    fn from(ser: ClaimsSerialized) -> Self {
        Self {
            aud: ser.aud,
            sub: ser.sub,
            scope: ser.scope.split(' ').map(|s| s.to_string()).collect(),
        }
    }
}
