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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RegisterUserParams {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginUserParams {
    pub username: String,
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
}

impl ApiClient {
    pub async fn register(&self, params: RegisterUserParams) -> FrontendResult<LocalUserView> {
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

    pub async fn get_person_edits(&self, person_id: PersonId) -> FrontendResult<Vec<EditView>> {
        let data = GetEditList {
            person_id: Some(person_id),
            ..Default::default()
        };
        self.get("/api/v1/edit/list", Some(data)).await
    }
}
