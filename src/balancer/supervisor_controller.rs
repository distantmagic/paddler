use actix_ws::Session;

pub struct SupervisorController {
    pub id: String,
    pub name: Option<String>,
    pub session: Session,
}
