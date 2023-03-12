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
use crate::post::post_08_simple_queries::{add_document_to_simple_query_table, delete_document_from_simple_query_table};

use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::field_value;
use crate::utils::{field_value_proto_to_sql, prepare_field_value_constraint};


pub fn get_matching_simple_query_subscriptions(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document: &Document) -> Vec<String>
{
  let operator_pairs = vec![("<", ">"), ("<=", ">="), ("=", "="), ("!=", "!="), (">", "<"), (">=", "<=")];

  let mut matching_subscriptions = vec![];
  for (field_name, field_value) in document.fields.iter() {
    let sql_field_value = field_value_proto_to_sql(field_value);
    for operator_pair in &operator_pairs {
      let collection_query = format!("select subscription_id from simple_query_subscriptions where collection_parent_path = $1 and collection_id = $2 and field_name = $3 and field_operator = $4 and field_value {} $5", operator_pair.1);
      let collection_subscriptions = transaction.query(
        &collection_query,
        &[&collection_parent_path, &collection_id, &field_name, &operator_pair.0, &sql_field_value],
      ).unwrap().into_iter().map(|x| x.get::<usize, String>(0));
      matching_subscriptions.extend(collection_subscriptions);

      let collection_group_query = format!("select subscription_id from simple_query_subscriptions where collection_parent_path IS NULL and collection_id = $1 and field_name = $2 and field_operator = $3 and field_value {} $4", operator_pair.1);
      let collection_group_subscriptions = transaction.query(
        &collection_group_query,
        &[&collection_id, &field_name, &operator_pair.0, &sql_field_value],
      ).unwrap().into_iter().map(|x| x.get::<usize, String>(0));
      matching_subscriptions.extend(collection_group_subscriptions)
    }
  }
  matching_subscriptions
}



// =================================================================================================
// =================================================================================================
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
  matching_subscriptions.extend(get_matching_simple_query_subscriptions(transaction, collection_parent_path, collection_id, document).into_iter());

  // Todo: send update to matching subscriptions
}

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
    matching_subscriptions.extend(get_matching_simple_query_subscriptions(transaction, collection_parent_path, collection_id, &document).into_iter());

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

  // Todo: trigger first subscription update?
  subscription_id
}
