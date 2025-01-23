use super::{result_to_option, ApiClient};
use crate::common::{
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
};
use leptos::prelude::ServerFnError;

impl ApiClient {
    pub async fn register(
        &self,
        params: RegisterUserParams,
    ) -> Result<LocalUserView, ServerFnError> {
        self.post("/api/v1/account/register", Some(&params)).await
    }

    pub async fn login(&self, params: LoginUserParams) -> Result<LocalUserView, ServerFnError> {
        self.post("/api/v1/account/login", Some(&params)).await
    }

    pub async fn logout(&self) -> Option<SuccessResponse> {
        result_to_option(self.post("/api/v1/account/logout", None::<()>).await)
    }

    pub async fn get_user(&self, data: GetUserParams) -> Option<DbPerson> {
        self.get("/api/v1/user", Some(data)).await
    }

    pub async fn update_user_profile(
        &self,
        data: UpdateUserParams,
    ) -> Result<SuccessResponse, ServerFnError> {
        self.post("/api/v1/account/update", Some(data)).await
    }

    pub async fn get_person_edits(&self, person_id: PersonId) -> Option<Vec<EditView>> {
        let data = GetEditList {
            person_id: Some(person_id),
            ..Default::default()
        };
        self.get("/api/v1/edit/list", Some(data)).await
    }
}
