use crate::auth::AuthenticatedUser;
use crate::entities::contacts;
use askama::Template;
use askama_web::WebTemplate;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{FromForm, State};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

#[derive(Template, WebTemplate)]
#[template(path = "contacts.j2")]
pub struct ContactsTemplate {
    pub username: String,
    pub role: String,
    pub contacts: Vec<contacts::Model>,
    pub editing_contact: Option<contacts::Model>,
}

#[derive(FromForm)]
pub struct ContactForm<'r> {
    pub name: &'r str,
    pub latitude: f64,
    pub longitude: f64,
    pub contact_type: &'r str,
}

#[rocket::get("/crud/contacts?<edit>")]
pub async fn list_contacts(
    edit: Option<i32>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<ContactsTemplate, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let contacts = contacts::Entity::find()
        .all(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let editing_contact = match edit {
        Some(id) => contacts::Entity::find_by_id(id)
            .one(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?,
        None => None,
    };

    Ok(ContactsTemplate {
        username: user.username,
        role: user.role,
        contacts,
        editing_contact,
    })
}

#[rocket::post("/crud/contacts", data = "<form_data>")]
pub async fn create_contact(
    form_data: Form<ContactForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
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

    new_contact
        .insert(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to("/crud/contacts"))
}

#[rocket::put("/crud/contacts/<id>", data = "<form_data>")]
pub async fn update_contact(
    id: i32,
    form_data: Form<ContactForm<'_>>,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    let contact = contacts::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if let Some(c) = contact {
        let contact_type = match form_data.contact_type {
            "Supplier" => crate::entities::sea_orm_active_enums::ContactType::Supplier,
            _ => crate::entities::sea_orm_active_enums::ContactType::Consumer,
        };

        let mut active: contacts::ActiveModel = c.into();
        active.name = Set(form_data.name.to_string());
        active.latitude = Set(form_data.latitude);
        active.longitude = Set(form_data.longitude);
        active.contact_type = Set(contact_type);
        active.updated_at = Set(chrono::Utc::now().naive_utc());
        active.update(db.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(Redirect::to("/crud/contacts"))
}

#[rocket::delete("/crud/contacts/<id>")]
pub async fn delete_contact(
    id: i32,
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Redirect, Status> {
    if user.role != "Admin" {
        return Err(Status::Forbidden);
    }

    contacts::Entity::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::to("/crud/contacts"))
}
