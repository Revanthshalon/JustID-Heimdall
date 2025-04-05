use crate::persistance::schema::RelationTuple as DbRelationTuple;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct RelationTuple {
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
    pub subject: Subject,
}

#[derive(Debug, Serialize, Clone)]
#[allow(unused)]
pub enum Subject {
    Direct(SubjectID),
    Set(SubjectSet),
}

#[derive(Debug, Serialize, Clone)]
pub struct SubjectID {
    pub id: Uuid,
}

#[allow(unused)]
impl SubjectID {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    pub fn unique_id(&self) -> Uuid {
        self.id
    }

    pub fn equals(&self, other: Subject) -> bool {
        match other {
            Subject::Set(_) => false,
            Subject::Direct(SubjectID { id }) => self.id.eq(&id),
        }
    }
}

impl std::fmt::Display for SubjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct SubjectSet {
    pub namespace: String,
    pub object: Uuid,
    pub relation: String,
}

#[allow(unused)]
impl SubjectSet {
    pub fn new(namespace: String, object: Uuid, relation: String) -> Self {
        Self {
            namespace,
            object,
            relation,
        }
    }

    pub fn unique_id(&self) -> Uuid {
        let namespace_relation = format!("{}-{}", self.namespace, self.relation);
        Uuid::new_v5(&self.object, namespace_relation.as_bytes())
    }

    pub fn equals(&self, other: Subject) -> bool {
        match other {
            Subject::Direct(_) => false,
            Subject::Set(SubjectSet {
                namespace,
                object,
                relation,
            }) => {
                self.namespace.eq(&namespace)
                    && self.object.eq(&object)
                    && self.relation.eq(&relation)
            }
        }
    }
}

impl std::fmt::Display for SubjectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:{}#{}", self.namespace, self.object, self.relation)
    }
}

impl From<DbRelationTuple> for RelationTuple {
    fn from(value: DbRelationTuple) -> Self {
        let subject = if let Some(id) = value.subject_id {
            Subject::Direct(SubjectID { id })
        } else {
            Subject::Set(SubjectSet {
                namespace: value.namespace.clone(),
                object: value.object,
                relation: value.relation.clone(),
            })
        };
        Self {
            namespace: value.namespace.clone(),
            object: value.object,
            relation: value.relation.clone(),
            subject,
        }
    }
}
