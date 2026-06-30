use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contacts::Table)
                    .if_not_exists()
                    .col(pk_auto(Contacts::ContactId))
                    .col(string(Contacts::Name))
                    .col(ColumnDef::new(Contacts::Latitude).double().not_null())
                    .col(ColumnDef::new(Contacts::Longitude).double().not_null())
                    .col(
                        ColumnDef::new(Contacts::ContactType)
                            .enumeration(
                                Alias::new("contact_type"),
                                [Alias::new("Consumer"), Alias::new("Supplier")],
                            )
                            .not_null(),
                    )
                    .col(date_time(Contacts::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(Contacts::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(Contacts::DeletedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-contacts-contact_type")
                    .table(Contacts::Table)
                    .col(Contacts::ContactType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-contacts-contact_type")
                    .table(Contacts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Contacts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    ContactId,
    Name,
    Latitude,
    Longitude,
    ContactType,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
