use leptos::*;
use leptos_form::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Default, Form, Serialize)]
#[form(
    component(
        action = create_my_data(my_data),
        on_success = |DbMyData { id, .. }, _| view!(<div>{format!("Created {id}")}</div>)
    )
)
]
pub struct MyData {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbMyData {
    pub id: i32,
    pub name: String,
}

async fn create_my_data(my_data: MyData) -> Result<DbMyData, ServerFnError> {
    info!("{:?}", &my_data);
    Ok(DbMyData {
        id: 1,
        name: my_data.username,
    })
}

#[component]
pub fn Login() -> impl IntoView {
    view! {
        <MyData
            initial={MyData::default()}
            top=|| view!(<input type="button" value="Login" />)
        />
    }
}
