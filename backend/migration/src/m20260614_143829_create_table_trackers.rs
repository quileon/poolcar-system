use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Trackers::Table)
                    .if_not_exists()
                    .col(pk_auto(Trackers::TrackerId))
                    .col(string(Trackers::Name))
                    .col(date_time(Trackers::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(Trackers::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(Trackers::DeletedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Trackers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Trackers {
    Table,
    TrackerId,
    Name,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
