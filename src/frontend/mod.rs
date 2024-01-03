pub mod api;
pub mod app;
pub mod article;
pub mod nav;

use leptos::error::Result;
use leptos::*;
use log::info;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cat {
    url: String,
}

type CatCount = usize;

async fn fetch_cats(count: CatCount) -> Result<Vec<String>> {
    if count > 0 {
        // make the request
        let res = reqwest::get(&format!(
            "https://api.thecatapi.com/v1/images/search?limit={count}",
        ))
        .await?
        .json::<Vec<Cat>>()
        .await?
        // extract the URL field for each cat
        .into_iter()
        .take(count)
        .map(|cat| cat.url)
        .collect::<Vec<_>>();
        Ok(res)
    } else {
        Ok(vec![])
    }
}

// TODO: import this from backend somehow
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DbInstance {
    pub id: i32,
    pub ap_id: Url,
    pub articles_url: Url,
    pub inbox_url: String,
    #[serde(skip)]
    pub public_key: String,
    #[serde(skip)]
    pub private_key: Option<String>,
    pub local: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InstanceView {
    pub instance: DbInstance,
    pub following: Vec<DbInstance>,
}

async fn fetch_instance(url: &str) -> Result<InstanceView> {
    let res = reqwest::get(url).await?.json::<InstanceView>().await?;
    info!("{:?}", &res);
    Ok(res)
}

pub fn fetch_example() -> impl IntoView {
    let (cat_count, set_cat_count) = create_signal::<CatCount>(0);

    // we use local_resource here because
    // 1) our error type isn't serializable/deserializable
    // 2) we're not doing backend-side rendering in this example anyway
    //    (during SSR, create_resource will begin loading on the backend and resolve on the client)
    let cats = create_local_resource(move || cat_count.get(), fetch_cats);
    //let instance = create_local_resource(move || "http://localhost:8131/api/v1/instance", fetch_instance);

    let fallback = move |errors: RwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                    .collect_view()
            })
        };

        view! {
            <div class="error">
                <h2>"Error"</h2>
                <ul>{error_list}</ul>
            </div>
        }
    };

    // the renderer can handle Option<_> and Result<_> states
    // by displaying nothing for None if the resource is still loading
    // and by using the ErrorBoundary fallback to catch Err(_)
    // so we'll just use `.and_then()` to map over the happy path
    let cats_view = move || {
        cats.and_then(|data| {
            data.iter()
                .map(|s| view! { <p><img src={s}/></p> })
                .collect_view()
        })
    };

    /*
    let instance_view = move || {
        instance.and_then(|data| {
            view! { <h1>{data.instance.ap_id.to_string()}</h1> }
        })
    };
     */

    view! {
        //{instance_view}
        <div>
            <label>
                "How many cats would you like?"
                <input
                    type="number"
                    prop:value=move || cat_count.get().to_string()
                    on:input=move |ev| {
                        let val = event_target_value(&ev).parse::<CatCount>().unwrap_or(0);
                        set_cat_count.set(val);
                    }
                />
            </label>
            <Transition fallback=move || {
                view! { <div>"Loading (Suspense Fallback)..."</div> }
            }>
                <ErrorBoundary fallback>
                <div>
                    {cats_view}
                </div>
                </ErrorBoundary>
            </Transition>
        </div>
    }
}
