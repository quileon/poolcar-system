use crate::auth::AuthenticatedUser;
use crate::entities::contacts;
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{FromForm, State};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set};

#[derive(Template, WebTemplate)]
#[template(path = "contacts.j2")]
pub struct ContactsTemplate {
    pub username: String,
    pub role: String,
    pub contacts: Vec<contacts::Model>,
    pub editing_contact: Option<contacts::Model>,
    pub current_page: u64,
    pub total_pages: u64,
    pub pages: Vec<u64>,
    pub error: Option<String>,
}

#[derive(FromForm)]
pub struct ContactForm<'r> {
    pub name: &'r str,
    pub latitude: f64,
    pub longitude: f64,
    pub contact_type: &'r str,
}

async fn render_contacts(
    db: &DatabaseConnection,
    user: &AuthenticatedUser,
    edit: Option<i32>,
    page: Option<u64>,
    error: Option<String>,
) -> Result<ContactsTemplate, Status> {
    let current_page = page.unwrap_or(1);
    let page_size = 5;

    let paginator = contacts::Entity::find()
        .paginate(db, page_size);

    let raw_total_pages = paginator.num_pages().await.map_err(|_| Status::InternalServerError)?;
    let total_pages = std::cmp::max(1, raw_total_pages);
    let target_page = std::cmp::min(current_page, total_pages);

    let contacts = paginator
        .fetch_page(target_page.saturating_sub(1))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let editing_contact = match edit {
        Some(id) => contacts::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    let pages = (1..=total_pages).collect::<Vec<u64>>();

    Ok(ContactsTemplate {
        username: user.username.clone(),
        role: user.role.clone(),
        contacts,
        editing_contact,
        current_page: target_page,
        total_pages,
        pages,
        error,
    })
}

#[rocket::get("/crud/contacts?<edit>&<page>")]
pub async fn list_contacts(
    edit: Option<i32>,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<ContactsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }
    render_contacts(db.inner(), &user, edit, page, None).await
}

#[rocket::post("/crud/contacts", data = "<form_data>")]
pub async fn create_contact(
    form_data: Form<ContactForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<ContactsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let now = chrono::Utc::now().naive_utc();
    let contact_type = match form_data.contact_type {
        "Supplier" => crate::entities::sea_orm_active_enums::ContactType::Supplier,
        _ => crate::entities::sea_orm_active_enums::ContactType::Consumer,
    };

    let new_contact = contacts::ActiveModel {
        name: Set(form_data.name.to_string()),
        latitude: Set(form_data.latitude),
        longitude: Set(form_data.longitude),
        contact_type: Set(contact_type),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    match new_contact.insert(db.inner()).await {
        Ok(_) => render_contacts(db.inner(), &user, None, None, None).await,
        Err(err) => render_contacts(db.inner(), &user, None, None, Some(err.to_string())).await,
    }
}

#[rocket::put("/crud/contacts/<id>?<page>", data = "<form_data>")]
pub async fn update_contact(
    id: i32,
    page: Option<u64>,
    form_data: Form<ContactForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<ContactsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let contact = match contacts::Entity::find_by_id(id).one(db.inner()).await {
        Ok(Some(c)) => c,
        Ok(None) => return render_contacts(db.inner(), &user, Some(id), page, Some(format!("Contact with ID {} not found in the database.", id))).await,
        Err(err) => return render_contacts(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    };

    let contact_type = match form_data.contact_type {
        "Supplier" => crate::entities::sea_orm_active_enums::ContactType::Supplier,
        _ => crate::entities::sea_orm_active_enums::ContactType::Consumer,
    };

    let mut active: contacts::ActiveModel = contact.into();
    active.name = Set(form_data.name.to_string());
    active.latitude = Set(form_data.latitude);
    active.longitude = Set(form_data.longitude);
    active.contact_type = Set(contact_type);
    active.updated_at = Set(chrono::Utc::now().naive_utc());

    match active.update(db.inner()).await {
        Ok(_) => render_contacts(db.inner(), &user, None, page, None).await,
        Err(err) => render_contacts(db.inner(), &user, Some(id), page, Some(err.to_string())).await,
    }
}

#[rocket::delete("/crud/contacts/<id>?<page>")]
pub async fn delete_contact(
    id: i32,
    page: Option<u64>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<ContactsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    match contacts::Entity::delete_by_id(id).exec(db.inner()).await {
        Ok(_) => render_contacts(db.inner(), &user, None, page, None).await,
        Err(err) => render_contacts(db.inner(), &user, None, page, Some(err.to_string())).await,
    }
}
