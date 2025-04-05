use uuid::Uuid;

use crate::models::relation_tuple::Subject;

#[allow(unused)]
pub struct RelationTupleQuery {
    pub namespace: Option<String>,
    pub object: Option<Uuid>,
    pub relation: Option<String>,
    pub subject: Option<Subject>,
}
