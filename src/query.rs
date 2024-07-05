use crate::document::{Document, Value};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    In,
    Nin,
}

#[derive(Debug, Clone)]
pub struct Condition {
    field: String,
    operator: Operator,
    value: Value,
}

#[derive(Debug, Clone)]
pub struct Query {
    conditions: Vec<Condition>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn add_condition(&mut self, field: impl Into<String>, operator: Operator, value: Value) {
        self.conditions.push(Condition {
            field: field.into(),
            operator,
            value,
        });
    }
}

pub struct QueryExecutor;

impl QueryExecutor {
    pub fn execute<'a, 'b>(
        &'a self,
        query: &'b Query,
        documents: &'a [Document],
    ) -> Vec<&'a Document> {
        documents
            .iter()
            .filter(|doc| self.matches_all_conditions(doc, &query.conditions))
            .collect()
    }

    fn matches_all_conditions(&self, doc: &Document, conditions: &[Condition]) -> bool {
        conditions
            .iter()
            .all(|condition| self.matches_condition(doc, condition))
    }

    fn matches_condition(&self, doc: &Document, condition: &Condition) -> bool {
        if let Some(doc_value) = doc.get(&condition.field) {
            match &condition.operator {
                Operator::Eq => doc_value == &condition.value,
                Operator::Ne => doc_value != &condition.value,
                Operator::Gt => {
                    self.compare_values(doc_value, &condition.value) == Some(Ordering::Greater)
                }
                Operator::Lt => {
                    self.compare_values(doc_value, &condition.value) == Some(Ordering::Less)
                }
                Operator::Gte => {
                    let ordering = self.compare_values(doc_value, &condition.value);
                    ordering == Some(Ordering::Greater) || ordering == Some(Ordering::Equal)
                }
                Operator::Lte => {
                    let ordering = self.compare_values(doc_value, &condition.value);
                    ordering == Some(Ordering::Less) || ordering == Some(Ordering::Equal)
                }
                Operator::In => {
                    if let Value::Array(array) = &condition.value {
                        array.contains(doc_value)
                    } else {
                        false
                    }
                }
                Operator::Nin => {
                    if let Value::Array(array) = &condition.value {
                        !array.contains(doc_value)
                    } else {
                        true
                    }
                }
            }
        } else {
            false
        }
    }

    fn compare_values(&self, a: &Value, b: &Value) -> Option<Ordering> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
