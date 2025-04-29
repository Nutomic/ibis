use super::IbisContext;
use crate::{DbUrl, error::BackendResult};
use chrono::{DateTime, Utc};
use diesel::{
    Identifiable,
    Insertable,
    QueryDsl,
    Queryable,
    RunQueryDsl,
    Selectable,
    dsl::insert_into,
};
use ibis_database_schema::sent_activity;
use std::ops::DerefMut;

#[derive(Insertable, Queryable, Selectable, Identifiable)]
#[diesel(table_name = sent_activity, check_for_backend(diesel::pg::Pg))]
pub struct SentActivity {
    pub id: DbUrl,
    pub json: String,
    pub published: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = sent_activity, check_for_backend(diesel::pg::Pg))]
pub struct SentActivityInsertForm {
    pub id: DbUrl,
    pub json: String,
}

impl SentActivity {
    pub fn create(form: SentActivityInsertForm, context: &IbisContext) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        insert_into(sent_activity::table)
            .values(form)
            .execute(conn.deref_mut())?;
        Ok(())
    }

    pub fn read(id: DbUrl, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(sent_activity::table.find(id).get_result(conn.deref_mut())?)
    }
}
