use rocket::http::Header;

#[derive(rocket::Responder)]
#[response(status = 200)]
pub struct HxRedirect {
    pub body: &'static str,
    pub header: Header<'static>,
}

#[derive(rocket::Responder)]
#[response(status = 200)]
pub struct HxLocation {
    pub body: &'static str,
    pub header: Header<'static>,
}
