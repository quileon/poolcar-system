use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Activities::Table)
                    .if_not_exists()
                    .col(pk_auto(Activities::ActivityId))
                    .col(integer_null(Activities::CarId))
                    .col(integer(Activities::ContactId))
                    .col(
                        ColumnDef::new(Activities::ActivityType)
                            .enumeration(
                                Alias::new("activity_type"),
                                [
                                    Alias::new("Delivery"),
                                    Alias::new("Meeting"),
                                    Alias::new("Trial T1"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(integer_null(Activities::TrackerId))
                    .col(date_time_null(Activities::StartedAt).default(Expr::current_timestamp()))
                    .col(date_time_null(Activities::FinishedAt))
                    .col(ColumnDef::new(Activities::FinishedLatitude).double())
                    .col(ColumnDef::new(Activities::FinishedLongitude).double())
                    .col(text_null(Activities::Description))
                    .col(date_time(Activities::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(Activities::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(Activities::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activities-car_id")
                            .from(Activities::Table, Activities::CarId)
                            .to(Cars::Table, Cars::CarId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activities-contact_id")
                            .from(Activities::Table, Activities::ContactId)
                            .to(Contacts::Table, Contacts::ContactId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activities-tracker_id")
                            .from(Activities::Table, Activities::TrackerId)
                            .to(Trackers::Table, Trackers::TrackerId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx-activities-car_id")
                    .table(Activities::Table)
                    .col(Activities::CarId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activities-contact_id")
                    .table(Activities::Table)
                    .col(Activities::ContactId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activities-activity_type")
                    .table(Activities::Table)
                    .col(Activities::ActivityType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activities-tracker_id")
                    .table(Activities::Table)
                    .col(Activities::TrackerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-activities-finished_at")
                    .table(Activities::Table)
                    .col(Activities::FinishedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-activities-finished_at")
                    .table(Activities::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-activities-tracker_id")
                    .table(Activities::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-activities-activity_type")
                    .table(Activities::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-activities-contact_id")
                    .table(Activities::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-activities-car_id")
                    .table(Activities::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Activities::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Activities {
    Table,
    ActivityId,
    CarId,
    ContactId,
    ActivityType,
    TrackerId,
    StartedAt,
    FinishedAt,
    FinishedLatitude,
    FinishedLongitude,
    Description,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Cars {
    Table,
    CarId,
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    ContactId,
}

#[derive(DeriveIden)]
enum Trackers {
    Table,
    TrackerId,
}
