use super::ApiClient;
use crate::{article::GetEditList, errors::FrontendResult};
use ibis_database::common::{
    SuccessResponse,
    article::EditView,
    instance::InstanceFollow,
    newtypes::PersonId,
    user::{LocalUserView, Person},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RegisterUserParams {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginUserParams {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetUserParams {
    pub name: String,
    pub domain: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateUserParams {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Logging in with an OAuth 2.0 authorization
pub struct AuthenticateWithOauth {
    pub code: String,
    pub oauth_issuer: Url,
    pub redirect_uri: Url,
    /// Username is mandatory at registration time
    pub username: Option<String>,
    pub pkce_code_verifier: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Response from OAuth token endpoint
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistrationResponse {
    pub user: LocalUserView,
    pub email_verification_required: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VerifyEmailParams {
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChangePasswordParams {
    pub new_password: String,
    pub confirm_new_password: String,
    pub old_password: String,
}

impl ApiClient {
    pub async fn register(
        &self,
        params: RegisterUserParams,
    ) -> FrontendResult<RegistrationResponse> {
        self.post("/api/v1/account/register", Some(&params)).await
    }

    pub async fn login(&self, params: LoginUserParams) -> FrontendResult<LocalUserView> {
        self.post("/api/v1/account/login", Some(&params)).await
    }

    pub async fn logout(&self) -> FrontendResult<SuccessResponse> {
        self.post("/api/v1/account/logout", None::<()>).await
    }

    pub async fn get_user(&self, data: GetUserParams) -> FrontendResult<Person> {
        self.get("/api/v1/user", Some(data)).await
    }

    pub async fn get_follows(&self) -> FrontendResult<Vec<InstanceFollow>> {
        self.get("/api/v1/user/follows", None::<()>).await
    }

    pub async fn update_user_profile(
        &self,
        data: UpdateUserParams,
    ) -> FrontendResult<SuccessResponse> {
        self.post("/api/v1/account/update", Some(data)).await
    }
    pub async fn change_password(
        &self,
        data: ChangePasswordParams,
    ) -> FrontendResult<SuccessResponse> {
        self.post("/api/v1/account/change_password", Some(data))
            .await
    }

    pub async fn get_person_edits(&self, person_id: PersonId) -> FrontendResult<Vec<EditView>> {
        let data = GetEditList {
            person_id: Some(person_id),
            ..Default::default()
        };
        self.get("/api/v1/edit/list", Some(data)).await
    }

    pub async fn verify_email(&self, token: String) -> FrontendResult<SuccessResponse> {
        let params = VerifyEmailParams { token };
        self.post("/api/v1/account/verify_email", Some(params))
            .await
    }
}
