pub use sea_orm_migration::prelude::*;

mod m20260614_143829_create_table_trackers;
mod m20260614_151745_create_table_cars;
mod m20260616_015527_create_table_contacts;
mod m20260616_032844_create_table_activities;
mod m20260616_033108_create_table_users;
mod m20260616_033121_create_table_car_status;
mod m20260616_033125_create_table_audit;
mod m20260616_033445_create_table_hardware_test;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260614_143829_create_table_trackers::Migration),
            Box::new(m20260614_151745_create_table_cars::Migration),
            Box::new(m20260616_015527_create_table_contacts::Migration),
            Box::new(m20260616_032844_create_table_activities::Migration),
            Box::new(m20260616_033108_create_table_users::Migration),
            Box::new(m20260616_033121_create_table_car_status::Migration),
            Box::new(m20260616_033125_create_table_audit::Migration),
            Box::new(m20260616_033445_create_table_hardware_test::Migration),
        ]
    }
}
