use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::UserId))
                    .col(string(Users::Username).unique_key())
                    .col(string(Users::Email).unique_key())
                    .col(string(Users::Password))
                    .col(string(Users::FullName))
                    .col(
                        ColumnDef::new(Users::UserRole)
                            .enumeration(
                                Alias::new("user_role"),
                                [Alias::new("Admin"), Alias::new("Security"), Alias::new("Employee")],
                            )
                            .not_null(),
                    )
                    .col(date_time(Users::CreatedAt).default(Expr::current_timestamp()))
                    .col(
                        date_time(Users::UpdatedAt)
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP".to_owned()),
                    )
                    .col(date_time_null(Users::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx-users-user_role")
                    .table(Users::Table)
                    .col(Users::UserRole)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-users-username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-users-email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-users-full_name")
                    .table(Users::Table)
                    .col(Users::FullName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-users-full_name")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-users-email")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-users-username")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-users-user_role")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
    Username,
    Email,
    Password,
    FullName,
    UserRole,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
