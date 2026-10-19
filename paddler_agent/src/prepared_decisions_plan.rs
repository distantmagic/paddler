use crate::prepared_decisions::PreparedDecisions;
use crate::served_decisions::ServedDecisions;

pub enum PreparedDecisionsPlan {
    Keep(ServedDecisions),
    Start(PreparedDecisions),
}
