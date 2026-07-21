use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cars::Table)
                    .if_not_exists()
                    .col(pk_auto(Cars::CarId))
                    .col(string(Cars::Name))
                    .col(string(Cars::PoliceNumber).unique_key())
                    .col(boolean(Cars::Active).default(true))
                    .col(
                        ColumnDef::new(Cars::CarType)
                            .enumeration(
                                Alias::new("car_type"),
                                [Alias::new("Delivery"), Alias::new("Passenger")],
                            )
                            .not_null(),
                    )
                    .col(integer_null(Cars::TrackerId).unique_key())
                    .col(date_time(Cars::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(Cars::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(Cars::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cars-tracker_id")
                            .from(Cars::Table, Cars::TrackerId)
                            .to(Trackers::Table, Trackers::TrackerId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-cars-car_type")
                    .table(Cars::Table)
                    .col(Cars::CarType)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-cars-tracker_id")
                    .table(Cars::Table)
                    .col(Cars::TrackerId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-cars-police_number")
                    .table(Cars::Table)
                    .col(Cars::PoliceNumber)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-cars-car_type")
                    .table(Cars::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-cars-tracker_id")
                    .table(Cars::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-cars-police_number")
                    .table(Cars::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Cars::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Cars {
    Table,
    CarId,
    Name,
    PoliceNumber,
    Active,
    CarType,
    TrackerId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Trackers {
    Table,
    TrackerId,
}
