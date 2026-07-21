use crate::auth::AuthenticatedUser;
use crate::entities::users;
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{FromForm, State};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set, QueryOrder};

#[derive(Template, WebTemplate)]
#[template(path = "users.j2")]
pub struct UsersTemplate {
    pub username: String,
    pub role: String,
    pub users: Vec<users::Model>,
    pub editing_user: Option<users::Model>,
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
    pub error: Option<String>,
}

#[derive(FromForm)]
pub struct UserForm<'r> {
    pub username: &'r str,
    pub email: &'r str,
    pub password: Option<&'r str>,
    pub full_name: &'r str,
    pub user_role: &'r str,
}

async fn render_users(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
    error: Option<String>,
) -> Result<UsersTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let editing_user = match edit {
        Some(id) => users::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let paginator = users::Entity::find()
        .order_by_asc(users::Column::UserId)
        .paginate(db, page_size);

    let raw_total_pages = paginator.num_pages().await.map_err(|_| Status::InternalServerError)?;
    let total_pages = std::cmp::max(1, raw_total_pages);
    let target_page = std::cmp::min(current_page, total_pages);

    let users_list = paginator
        .fetch_page(target_page.saturating_sub(1))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let pages = (1..=total_pages).collect::<Vec<u64>>();

    Ok(UsersTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        users: users_list,
        editing_user,
        current_page: target_page,
        total_pages,
        pages,
        error,
    })
}

#[rocket::get("/crud/users?<edit>&<page>")]
pub async fn list_users(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<UsersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }
    render_users(db.inner(), &user, edit, page, None).await
}

#[rocket::post("/crud/users", data = "<form_data>")]
pub async fn create_user(
    form_data: Form<UserForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<UsersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let password = match form_data.password {
        Some(p) if !p.trim().is_empty() => p.to_string(),
        _ => return render_users(db.inner(), &user, None, None, Some("Password is required on creation.".to_string())).await,
    };

    let user_role = match form_data.user_role {
        "Admin" => crate::entities::sea_orm_active_enums::UserRole::Admin,
        "Security" => crate::entities::sea_orm_active_enums::UserRole::Security,
        _ => crate::entities::sea_orm_active_enums::UserRole::Employee,
    };

    let now = chrono::Utc::now().naive_utc();

    let new_user = users::ActiveModel {
        username: Set(form_data.username.to_string()),
        email: Set(form_data.email.to_string()),
        password: Set(password),
        full_name: Set(form_data.full_name.to_string()),
        user_role: Set(user_role),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_user.insert(db.inner()).await {
        Ok(_) => render_users(db.inner(), &user, None, None, None).await,
        Err(err) => render_users(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
}

#[rocket::put("/crud/users/<id>?<page>", data = "<form_data>")]
pub async fn update_user(
    id: i32,
    page: Option<u64>,
    form_data: Form<UserForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<UsersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let user_record = match users::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(u)) => u,
        Ok(None) => return render_users(db.inner(), &user, Some(id), page, Some(format!("User with ID {} not found.", id))).await,
        Err(err) => return render_users(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    };

    let user_role = match form_data.user_role {
        "Admin" => crate::entities::sea_orm_active_enums::UserRole::Admin,
        "Security" => crate::entities::sea_orm_active_enums::UserRole::Security,
        _ => crate::entities::sea_orm_active_enums::UserRole::Employee,
    };

    let mut active: users::ActiveModel = user_record.into();
    active.username = Set(form_data.username.to_string());
    active.email = Set(form_data.email.to_string());
    active.full_name = Set(form_data.full_name.to_string());
    active.user_role = Set(user_role);
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    if let Some(p) = form_data.password {
        if !p.trim().is_empty() {
            active.password = Set(p.to_string());
        }
    }

    match active.update(db.inner()).await {
        Ok(_) => render_users(db.inner(), &user, None, page, None).await,
        Err(err) => render_users(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

#[rocket::delete("/crud/users/<id>?<page>")]
pub async fn delete_user(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<UsersTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let target_user = match users::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(u)) => u,
        Ok(None) => return render_users(db.inner(), &user, None, page, Some(format!("User with ID {} not found.", id))).await,
        Err(err) => return render_users(db.inner(), &user, None, page, Some(err.to_string())).await,
    };

    if target_user.username == user.username {
        return render_users(db.inner(), &user, None, page, Some("You cannot delete your own user account.".to_string())).await;
    }

    match users::Entity::delete_by_id(id).exec(db.inner()).await {
        Ok(_) => render_users(db.inner(), &user, None, page, None).await,
        Err(err) => render_users(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}
