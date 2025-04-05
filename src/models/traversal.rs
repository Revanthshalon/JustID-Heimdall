#![allow(unused)]

use super::relation_tuple::RelationTuple;

pub struct TraversalResult {
    pub from: RelationTuple,
    pub to: RelationTuple,
    pub via: Traversal,
    pub found: bool,
}

pub enum Traversal {
    Unknown,
    SubjectSetExpand,
    ComputedUserset,
    TupleToUserset,
}

impl std::fmt::Display for Traversal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Traversal::Unknown => writeln!(f, "unknown"),
            Traversal::SubjectSetExpand => writeln!(f, "subject set expand"),
            Traversal::ComputedUserset => writeln!(f, "computed userset"),
            Traversal::TupleToUserset => writeln!(f, "tuple to userset"),
        }
    }
}
