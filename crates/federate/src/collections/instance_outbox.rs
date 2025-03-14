use crate::{
    activities::announce::AnnounceActivity,
    objects::instance::InstanceWrapper,
    routes::AnnouncableActivities,
};
use activitypub_federation::{
    config::Data,
    kinds::collection::OrderedCollectionType,
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Collection},
};
use futures::future::join_all;
use ibis_database::{
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOutbox {
    pub(crate) r#type: OrderedCollectionType,
    pub(crate) id: String,
    pub(crate) total_items: i32,
    pub(crate) ordered_items: Vec<AnnounceActivity>,
}

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityOutbox(());

#[async_trait::async_trait]
impl Collection for ApubCommunityOutbox {
    type Owner = InstanceWrapper;
    type DataType = IbisContext;
    type Kind = GroupOutbox;
    type Error = BackendError;

    async fn read_local(
        owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> BackendResult<Self::Kind> {
        let site = Site::read_local(&mut data.pool()).await?;

        let post_views = PostQuery {
            community_id: Some(owner.id),
            sort: Some(PostSortType::New),
            limit: Some(FETCH_LIMIT_MAX),
            ..Default::default()
        }
        .list(&site, &mut data.pool())
        .await?;

        let mut ordered_items = vec![];
        for post_view in post_views {
            // ignore errors, in particular if post creator was deleted
            if let Ok(create) = CreateOrUpdatePage::new(
                post_view.post.into(),
                &post_view.creator.into(),
                owner,
                CreateOrUpdateType::Create,
                data,
            )
            .await
            {
                let announcable = AnnouncableActivities::CreateOrUpdatePost(create);
                if let Ok(announce) = AnnounceActivity::new(announcable.try_into()?, owner, data) {
                    ordered_items.push(announce);
                }
            }
        }

        Ok(GroupOutbox {
            r#type: OrderedCollectionType::OrderedCollection,
            id: format!("{}/outbox", &owner.ap_id),
            total_items: ordered_items.len() as i32,
            ordered_items,
        })
    }

    async fn verify(
        group_outbox: &GroupOutbox,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> BackendResult<()> {
        Ok(())
    }

    async fn from_json(
        apub: Self::Kind,
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> BackendResult<Self> {
        let mut outbox_activities = apub.ordered_items;
        if outbox_activities.len() as i64 > FETCH_LIMIT_MAX {
            outbox_activities = outbox_activities
                .get(0..(FETCH_LIMIT_MAX as usize))
                .unwrap_or_default()
                .to_vec();
        }

        // We intentionally ignore errors here. This is because the outbox might contain posts from old
        // Lemmy versions, or from other software which we cant parse. In that case, we simply skip the
        // item and only parse the ones that work.
        // process items in parallel, to avoid long delay from fetch_site_metadata() and other
        // processing
        join_all(outbox_activities.into_iter().map(|activity| {
            async {
                // Receiving announce requires at least one local community follower for anti spam purposes.
                // This won't be the case for newly fetched communities, so we extract the inner activity
                // and handle it directly to bypass this check.
                let inner = activity.object.object(data).await.map(TryInto::try_into);
                if let Ok(Ok(AnnouncableActivities::CreateOrUpdatePost(inner))) = inner {
                    let verify = inner.verify(data).await;
                    if verify.is_ok() {
                        inner.receive(data).await.ok();
                    }
                }
            }
        }))
        .await;

        // This return value is unused, so just set an empty vec
        Ok(ApubCommunityOutbox(()))
    }
}
