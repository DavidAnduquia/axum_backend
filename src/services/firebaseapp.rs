use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tokio::{fs, sync::Mutex};

use crate::utils::errors::AppError;

const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";

#[derive(Debug, Deserialize)]
struct RawServiceAccount {
    project_id: Option<String>,
    private_key: String,
    client_email: String,
    token_uri: String,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: DateTime<Utc>,
}

pub struct FirebaseAppService {
    project_id: String,
    client_email: String,
    token_uri: String,
    signing_key: EncodingKey,
    client: Client,
    token_cache: Mutex<Option<CachedToken>>,
}

impl FirebaseAppService {
    pub async fn from_credentials_file<P: AsRef<Path>>(
        project_id_override: Option<String>,
        credentials_path: P,
    ) -> Result<Self, AppError> {
        let raw = fs::read_to_string(credentials_path.as_ref())
            .await
            .map_err(|err| {
                AppError::InternalServerError(
                    format!("No se pudo leer el archivo de credenciales FCM: {}", err).into(),
                )
            })?;

        let creds: RawServiceAccount = serde_json::from_str(&raw).map_err(|err| {
            AppError::InternalServerError(
                format!("Credenciales FCM inválidas o corruptas: {}", err).into(),
            )
        })?;

        let project_id = project_id_override
            .or_else(|| creds.project_id.clone())
            .ok_or_else(|| {
                AppError::InternalServerError(
                    "FCM project_id no configurado ni en .env ni en el JSON".into(),
                )
            })?;

        let private_key = creds.private_key.replace("\\n", "\n");
        let signing_key = EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|err| {
            AppError::InternalServerError(
                format!("No se pudo parsear la llave privada FCM: {}", err).into(),
            )
        })?;

        let client = Client::builder()
            .user_agent("backend-aula-fcm/0.1.0")
            .build()
            .map_err(|err| {
                AppError::InternalServerError(
                    format!("No se pudo crear el cliente HTTP para FCM: {}", err).into(),
                )
            })?;

        Ok(Self {
            project_id,
            client_email: creds.client_email,
            token_uri: creds.token_uri,
            signing_key,
            client,
            token_cache: Mutex::new(None),
        })
    }

    /// Returns a human-readable summary without exposing sensitive fields
    pub fn summary(&self) -> String {
        format!(
            "project_id={}, client_email={}, token_uri={}",
            self.project_id, self.client_email, self.token_uri
        )
    }

    async fn get_access_token(&self) -> Result<String, AppError> {
        let mut cache = self.token_cache.lock().await;
        if let Some(token) = cache.as_ref() {
            if token.expires_at > Utc::now() + Duration::seconds(30) {
                return Ok(token.access_token.clone());
            }
        }

        let refreshed = self.request_access_token().await?;
        let token_value = refreshed.access_token.clone();
        *cache = Some(refreshed);
        Ok(token_value)
    }

    async fn request_access_token(&self) -> Result<CachedToken, AppError> {
        #[derive(Serialize)]
        struct GoogleClaims<'a> {
            iss: &'a str,
            scope: &'a str,
            aud: &'a str,
            exp: usize,
            iat: usize,
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: i64,
            token_type: String,
        }

        let now = Utc::now();
        let claims = GoogleClaims {
            iss: &self.client_email,
            scope: FCM_SCOPE,
            aud: &self.token_uri,
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(55)).timestamp() as usize,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        let assertion = encode(&header, &claims, &self.signing_key).map_err(|err| {
            AppError::InternalServerError(
                format!("No se pudo firmar el JWT para OAuth FCM: {}", err).into(),
            )
        })?;

        let response = self
            .client
            .post(&self.token_uri)
            .form(&[
                (
                    "grant_type",
                    "urn:ietf:params:oauth:grant-type:jwt-bearer",
                ),
                ("assertion", assertion.as_str()),
            ])
            .send()
            .await
            .map_err(map_reqwest_err)?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::InternalServerError(
                format!("Error OAuth al solicitar token FCM ({}): {}", status, body).into(),
            ));
        }

        let token_response: TokenResponse = response.json().await.map_err(|err| {
            AppError::InternalServerError(
                format!("Respuesta OAuth de FCM inválida: {}", err).into(),
            )
        })?;

      
        let expires_in = token_response.expires_in.max(60) - 30; // margen de seguridad
        let expires_at = Utc::now() + Duration::seconds(expires_in);

        Ok(CachedToken {
            access_token: token_response.access_token,
            expires_at,
        })
    }

    pub async fn send_message(&self, message: FcmMessage) -> Result<(), AppError> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let payload = json!({ "message": message });

        let response = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_err)?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(AppError::InternalServerError(
            format!("Error enviando notificación FCM ({}): {}", status, body).into(),
        ))
    }
}

fn map_reqwest_err(err: reqwest::Error) -> AppError {
    AppError::InternalServerError(
        format!("Error de red comunicando con FCM: {}", err).into(),
    )
}

#[derive(Debug, Serialize)]
pub struct FcmNotification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FcmMessage {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<FcmNotification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Map<String, Value>>,
}

impl FcmMessage {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            notification: None,
            data: None,
        }
    }

    pub fn with_notification(mut self, notification: FcmNotification) -> Self {
        self.notification = Some(notification);
        self
    }

    pub fn with_data(mut self, data: Map<String, Value>) -> Self {
        self.data = Some(data);
        self
    }
}

impl FcmNotification {
    pub fn new(title: Option<String>, body: Option<String>) -> Self {
        Self { title, body }
    }
}

pub type FirebaseAppHandle = Arc<FirebaseAppService>;
