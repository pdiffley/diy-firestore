use std::collections::{HashMap, HashSet};
use prost::Message;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use std::error::Error;
use std::env;
use std::fmt;
use bytes::{Bytes, BytesMut};

use itertools::Itertools;
use uuid::Uuid;

use crate::protos::document_protos::Document;
use crate::protos::document_protos::FieldValue;
use crate::protos::document_protos::field_value::Value;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::{SqlFieldValue};
use crate::utils::{field_value_proto_to_sql, get_document_from_row_id};

// TODO: Add security check when updating subscription data

pub fn simple_query(
  transaction: &mut Transaction,
  user_id: &UserId,
  collection_parent_path: &Option<String>,
  collection_id: &str,
  field_name: &str,
  field_operator: &str,
  field_value: &FieldValue
) -> Vec<Document> {
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &collection_parent_path,
                                 collection_id, &None));
  }

  let sql_field_value = field_value_proto_to_sql(field_value);
  let query_result;
  if let Some(collection_parent_path) = collection_parent_path {
    query_result = transaction.query(
      "SELECT (collection_parent_path, collection_id, document_id) from simple_query_lookup where collection_parent_path = $1, collection_id = $2, field_name = $3, field_value $4 $5",
      &[&collection_parent_path, &collection_id, &field_name, &field_operator, &sql_field_value])
  } else {
    query_result = transaction.query(
      "SELECT (collection_parent_path, collection_id, document_id) from simple_query_lookup where collection_id = $1, field_name = $2, field_value $3 $4",
      &[&collection_id, &field_name, &field_operator, &sql_field_value])
  }
  query_result.unwrap().into_iter()
    .map(|row| get_document_from_row_id(transaction, user_id, row))
    .collect()
}


pub fn get_affected_simple_query_subscriptions(transaction: &mut Transaction, collection_parent_path: &str, collection_id: &str, document: &Document) -> Vec<String> {
  let operator_pairs = vec![("<", ">="), ("<=", ">"), ("=", "="), (">", "<="), (">=", "<")];

  let mut affected_subscriptions = vec![];
  for (field_name, field_value) in document.fields.iter() {
    let sql_field_value = field_value_proto_to_sql(field_value);
    for operator_pair in &operator_pairs {
      let collection_subscriptions = transaction.query(
        "select subscription_id from simple_query_subscriptions where collection_parent_path = $1, collection_id = $2, field_name = $3, field_operator = $4, field_value $5 $6",
        &[&collection_parent_path, &collection_id, &field_name, &operator_pair.0, &operator_pair.1, &sql_field_value]
      ).unwrap().into_iter().map(|x| x.get::<usize, String>(0));
      affected_subscriptions.extend(collection_subscriptions);

      let collection_group_subscriptions = transaction.query(
        "select subscription_id from simple_query_subscriptions where collection_parent_path = NULL, collection_id = $1, field_name = $2, operator = $3, field_value $4 $5",
        &[&collection_id, &field_name, &operator_pair.0, &operator_pair.1, &sql_field_value]
      ).unwrap().into_iter().map(|x| x.get::<usize, String>(0));
      affected_subscriptions.extend(collection_group_subscriptions)
    }
  }

  affected_subscriptions
}

pub fn add_document_to_simple_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
)
{
  for (field_name, field_value) in document.fields.iter() {
    let field_value = field_value_proto_to_sql(&field_value);
    transaction.execute(
      "insert into simple_query_lookup values ($1, $2, $3, $4, $5)",
      &[&collection_parent_path, &collection_id, &document_id, &field_name, &field_value]).unwrap();
  }
}

pub fn delete_document_from_simple_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
)
{
  transaction.execute(
    "delete from simple_query_lookup where collection_parent_path=$1, collection_id=$2, document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id]).unwrap();
}

pub fn subscribe_to_simple_query(
  transaction: &mut Transaction,
  client_id: &str,
  collection_parent_path: &Option<String>,
  collection_id: &str,
  field_name: &str,
  field_operator: &str,
  field_value: &SqlFieldValue
) {
  let subscription_id: String = Uuid::new_v4().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();

  let collection_parent_path_string: String;
  if let Some(collection_parent_path) = collection_parent_path {
    collection_parent_path_string = collection_parent_path.clone();
  } else {
    collection_parent_path_string = "NULL".to_owned();
  }

  transaction.execute("insert into simple_query_subscriptions values ($1, $2, $3, $4, $5, $6)",
                      &[&collection_parent_path_string, &collection_id, &field_name, &field_operator, &field_value, &subscription_id]).unwrap();

  // Todo: trigger first subscription update?
}
