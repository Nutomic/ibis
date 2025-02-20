use crate::backend::{database::DbPool, utils::error::BackendResult};
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

fn active_counts(pool: &DbPool) -> BackendResult<()> {
    info!("Updating active user count");
    let mut conn = pool.get()?;

    let rows = sql_query("update instance_stats set users_active_month = (select * from instance_stats_activity('1 month'))")
        .execute(&mut conn)?;
    debug_assert_eq!(1, rows);
    let rows = sql_query("update instance_stats set users_active_half_year = (select * from instance_stats_activity('6 months'))")
        .execute(&mut conn)?;
    debug_assert_eq!(1, rows);

    info!("Done with active user count");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::{IbisConfig, IbisContext};

    #[test]
    fn test_scheduled_tasks() -> BackendResult<()> {
        let context = IbisContext::init(IbisConfig::read()?, false)?;
        active_counts(&context.db_pool)?;
        Ok(())
    }
}
