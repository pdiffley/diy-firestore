use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;

use bytes::{Bytes, BytesMut};
use itertools::Itertools;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use prost::Message;
use uuid::Uuid;
use crate::post::post_04_initial_write_operations::{add_document_to_documents_table, delete_document_from_documents_table, get_document};
use crate::post::post_05_basic_subscriptions::get_matching_basic_subscription_ids;

use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::field_value;
use crate::utils::{field_value_proto_to_sql, prepare_field_value_constraint};

// =================================================================================================
// =================================================================================================
// =================================================================================================

pub fn simple_query(
  transaction: &mut Transaction,
  collection_parent_path: &Option<String>,
  collection_id: &str,
  field_name: &str,
  field_operator: &str,
  field_value: &field_value,
) -> Vec<Document> {
  let query_result;
  if let Some(collection_parent_path) = collection_parent_path {
    let query_string = format!(
      "SELECT collection_parent_path, collection_id, document_id
      from simple_query_lookup
      where collection_parent_path = $1 and collection_id = $2 and
      field_name = $3 and field_value {} $4", field_operator);
    query_result = transaction.query(
      &query_string,
      &[&collection_parent_path, &collection_id, &field_name, &field_value])
  } else {
    let query_string = format!(
      "SELECT collection_parent_path, collection_id, document_id
      from simple_query_lookup
      where collection_id = $1 and field_name = $2
      and field_value {} $3", field_operator);
    query_result = transaction.query(
      &query_string,
      &[&collection_id, &field_name, &field_value])
  }

  query_result.unwrap().into_iter()
    .map(|row| get_document_from_row_id(transaction, row))
    .collect()
}

fn get_document_from_row_id(transaction: &mut Transaction, document_id_row: Row) -> Document {
  get_document(
    transaction,
    document_id_row.get("collection_parent_path"),
    document_id_row.get("collection_id"),
    document_id_row.get("document_id"))
    .unwrap()
}


// =================================================================================================
// =================================================================================================
// =================================================================================================


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

// =================================================================================================

fn create_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  document: &Document,
) {
  let mut encoded_document: Vec<u8> = vec![];
  document.encode(&mut encoded_document).unwrap();

  add_document_to_documents_table(transaction, collection_parent_path, collection_id, document_id, update_id, &encoded_document);
  add_document_to_simple_query_table(transaction, collection_parent_path, collection_id, document_id, document);

  let mut matching_subscriptions = vec![];
  matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());

  // Todo: send update to matching subscriptions
}


// =================================================================================================
// =================================================================================================
// =================================================================================================


pub fn delete_document_from_simple_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
)
{
  transaction.execute(
    "delete from simple_query_lookup where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id]).unwrap();
}

// =================================================================================================

pub fn delete_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
) {
  if let Some(document) = get_document(transaction, collection_parent_path, collection_id, document_id) {
    delete_document_from_documents_table(transaction, collection_parent_path, collection_id, document_id);
    delete_document_from_simple_query_table(transaction, collection_parent_path, collection_id, document_id);

    let mut matching_subscriptions = vec![];
    matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());

    // Todo: send update to matching subscriptions
  }
}







pub fn subscribe_to_simple_query(
  transaction: &mut Transaction,
  client_id: &str,
  collection_parent_path: &Option<String>,
  collection_id: &str,
  field_name: &str,
  field_operator: &str,
  field_value: &field_value)
  -> String
{
  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
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

  // Todo: send client first batch of subscription data
  subscription_id
}
