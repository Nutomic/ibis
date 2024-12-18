use super::schema::instance_stats;
use crate::backend::{IbisData, MyResult};
use diesel::{query_dsl::methods::FindDsl, Queryable, RunQueryDsl, Selectable};
use std::ops::DerefMut;

#[derive(Queryable, Selectable)]
#[diesel(table_name = instance_stats, check_for_backend(diesel::pg::Pg))]
pub struct InstanceStats {
    pub id: i32,
    pub users: i32,
    pub users_active_month: i32,
    pub users_active_half_year: i32,
    pub articles: i32,
}

impl InstanceStats {
    pub fn read(data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(instance_stats::table.find(1).get_result(conn.deref_mut())?)
    }
}