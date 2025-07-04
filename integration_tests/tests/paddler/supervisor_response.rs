use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Supervisor {
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SupervisorsResponse {
    pub supervisors: Vec<Supervisor>,
}
