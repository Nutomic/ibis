use crate::backend::{database::DbPool, utils::error::MyResult};
use clokwerk::{Scheduler, TimeUnits};
use diesel::{sql_query, RunQueryDsl};
use log::{error, info};
use std::time::Duration;

pub fn start(pool: DbPool) {
    let mut scheduler = Scheduler::new();

    active_counts(&pool).inspect_err(|e| error!("{e}")).ok();
    scheduler.every(1.hour()).run(move || {
        active_counts(&pool).inspect_err(|e| error!("{e}")).ok();
    });

    let _ = scheduler.watch_thread(Duration::from_secs(60));
}

fn active_counts(pool: &DbPool) -> MyResult<()> {
    info!("Updating active user count");
    let mut conn = pool.get()?;

    sql_query("update instance_stats set users_active_month = (select * from instance_stats_activity('1 month'))")
        .execute(&mut conn)?;
    sql_query("update instance_stats set users_active_half_year = (select * from instance_stats_activity('6 months'))")
        .execute(&mut conn)?;

    info!("Done with active user count");
    Ok(())
}
