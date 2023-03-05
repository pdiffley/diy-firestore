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

use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::field_value;

// =================================================================================================
// =================================================================================================
// =================================================================================================

pub fn get_matching_basic_subscription_ids(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str) -> Vec<String>
{
  let document_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id
    from basic_subscriptions
    where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id],
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let collection_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id
    from basic_subscriptions
    where collection_parent_path=$1 and collection_id=$2 and document_id IS NULL",
    &[&collection_parent_path, &collection_id],
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let collection_group_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id
    from basic_subscriptions
    where collection_parent_path IS NULL and collection_id=$1 and document_id IS NULL",
    &[&collection_id],
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let all_matching_subscriptions: Vec<String> =
    document_subscriptions.into_iter()
      .chain(collection_subscriptions.into_iter())
      .chain(collection_group_subscriptions.into_iter())
      .collect();
  all_matching_subscriptions
}

// ================================================================================================
// ================================================================================================
// ================================================================================================

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

  let mut matching_subscriptions = vec![];
  matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());

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

    let mut matching_subscriptions = vec![];
    matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());

    // Todo: send update to matching subscriptions
  }
}


// ================================================================================================
// ================================================================================================
// ================================================================================================


pub fn subscribe_to_document(
  transaction: &mut Transaction,
  client_id: &str,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
) -> String
{
  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();
  transaction.execute("insert into basic_subscriptions values ($1, $2, $3, $4)",
                      &[&collection_parent_path, &collection_id, &document_id, &subscription_id]).unwrap();

  // Todo: send client first batch of subscription data
  subscription_id
}

pub fn subscribe_to_collection(
  transaction: &mut Transaction,
  client_id: &str,
  collection_parent_path: &str,
  collection_id: &str)
  -> String
{
  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();
  transaction.execute("insert into basic_subscriptions values ($1, $2, NULL, $3)",
                      &[&collection_parent_path, &collection_id, &subscription_id]).unwrap();

  // Todo: send client first batch of subscription data
  subscription_id
}

pub fn subscribe_to_collection_group(
  transaction: &mut Transaction,
  client_id: &str,
  collection_id: &str)
  -> String
{
  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();
  transaction.execute("insert into basic_subscriptions values (NULL, $1, NULL, $2)",
                      &[&collection_id, &subscription_id]).unwrap();

  // Todo: send client first batch of subscription data
  subscription_id
}


// ================================================================================================
// ================================================================================================
// ================================================================================================

// fn get_document(
//   transaction: &mut Transaction,
//   collection_parent_path: &str,
//   collection_id: &str,
//   document_id: &str)
//   -> Option<Document>
// {
//   let rows = transaction.query(
//     "SELECT document_data
//     from documents
//     where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
//     &[&collection_parent_path, &collection_id, &document_id],
//   ).unwrap();
//
//   if rows.len() == 0 {
//     return None;
//   }
//
//   let encoded_document: Vec<u8> = rows[0].get(0);
//   let document: Document = Document::decode(&encoded_document[..]).unwrap();
//   Some(document)
// }
