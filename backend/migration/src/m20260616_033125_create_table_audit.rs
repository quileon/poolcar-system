use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Audit::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Audit::AuditId)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(integer_null(Audit::CarId))
                    .col(integer(Audit::TrackerId))
                    .col(ColumnDef::new(Audit::Latitude).double().not_null())
                    .col(ColumnDef::new(Audit::Longitude).double().not_null())
                    .col(date_time(Audit::RecordedAt).default(Expr::current_timestamp()))
                    .col(date_time(Audit::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(Audit::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(Audit::DeletedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-audit-car_id")
                            .from(Audit::Table, Audit::CarId)
                            .to(Cars::Table, Cars::CarId)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-audit-tracker_id")
                            .from(Audit::Table, Audit::TrackerId)
                            .to(Trackers::Table, Trackers::TrackerId)
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
                    .name("idx-audit-car_id")
                    .table(Audit::Table)
                    .col(Audit::CarId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-audit-tracker_id")
                    .table(Audit::Table)
                    .col(Audit::TrackerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-audit-recorded_at")
                    .table(Audit::Table)
                    .col(Audit::RecordedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-audit-recorded_at")
                    .table(Audit::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-audit-tracker_id")
                    .table(Audit::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-audit-car_id")
                    .table(Audit::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Audit::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Audit {
    Table,
    AuditId,
    CarId,
    TrackerId,
    Latitude,
    Longitude,
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

#[derive(DeriveIden)]
enum Trackers {
    Table,
    TrackerId,
}
