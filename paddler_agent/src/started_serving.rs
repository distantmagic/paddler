use crate::served_model::ServedModel;

pub enum StartedServing {
    Cancelled,
    NothingToServe,
    Serving(Box<ServedModel>),
}
