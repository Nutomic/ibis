use super::ApiClient;
use crate::{
    common::{
        article::{EditView, GetEditList},
        newtypes::PersonId,
        user::{
            DbPerson,
            GetUserParams,
            LocalUserView,
            LoginUserParams,
            RegisterUserParams,
            UpdateUserParams,
        },
        SuccessResponse,
    },
    frontend::utils::errors::FrontendResult,
};

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

    pub async fn get_user(&self, data: GetUserParams) -> FrontendResult<DbPerson> {
        self.get("/api/v1/user", Some(data)).await
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
