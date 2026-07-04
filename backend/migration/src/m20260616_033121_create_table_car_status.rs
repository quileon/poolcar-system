use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CarStatus::Table)
                    .if_not_exists()
                    .col(pk_auto(CarStatus::CarStatusId))
                    .col(integer(CarStatus::CarId))
                    .col(ColumnDef::new(CarStatus::GasLevel).double().not_null())
                    .col(ColumnDef::new(CarStatus::Kilometres).double().not_null())
                    .col(
                        ColumnDef::new(CarStatus::StatusType)
                            .enumeration(
                                Alias::new("status_type"),
                                [Alias::new("Departure"), Alias::new("Return")],
                            )
                            .not_null(),
                    )
                    .col(date_time(CarStatus::RecordedAt).default(Expr::current_timestamp()))
                    .col(date_time(CarStatus::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(CarStatus::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(CarStatus::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-car_status-car_id")
                            .from(CarStatus::Table, CarStatus::CarId)
                            .to(Cars::Table, Cars::CarId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx-car_status-car_id")
                    .table(CarStatus::Table)
                    .col(CarStatus::CarId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-car_status-recorded_at")
                    .table(CarStatus::Table)
                    .col(CarStatus::RecordedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-car_status-type")
                    .table(CarStatus::Table)
                    .col(CarStatus::StatusType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-car_status-type")
                    .table(CarStatus::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-car_status-recorded_at")
                    .table(CarStatus::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-car_status-car_id")
                    .table(CarStatus::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(CarStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CarStatus {
    Table,
    CarStatusId,
    CarId,
    GasLevel,
    Kilometres,
    StatusType,
    RecordedAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Cars {
    Table,
    CarId,
}
