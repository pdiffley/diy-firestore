use std::collections::{HashMap, HashSet};
use prost::Message;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use std::error::Error;
use std::env;
use std::fmt;
use bytes::{Bytes, BytesMut};
use uuid::Uuid;

use itertools::Itertools;

use crate::protos::document_protos::Document;
use crate::protos::document_protos::FieldValue;
use crate::protos::document_protos::field_value::Value;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::{field_value};

pub fn get_document(
  transaction: &mut Transaction, 
  user_id: &UserId, 
  collection_parent_path: &str, 
  collection_id: &str, 
  document_id: &str) 
  -> Option<Document> 
{
  // security check
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::Get,
                                 &Some(collection_parent_path.to_owned()),
                                 collection_id, &Some(document_id.to_owned())));
  }

  let rows = transaction.query(
    "SELECT document_data 
    from documents 
    where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id]
  ).unwrap();

  if rows.len() == 0 {
    return None
  }

  let encoded_document: Vec<u8> = rows[0].get(0);

  let document: Document = Document::decode(&encoded_document[..]).unwrap();
  Some(document)
}

pub fn get_documents(
  transaction: &mut Transaction,
  user_id: &UserId,
  collection_parent_path: &str,
  collection_id: &str)
  -> Vec<Document>
{
  // list security check
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &Some(collection_parent_path.to_owned()),
                                 collection_id, &None));
  }

  let document_ids: Vec<String> = transaction.query(
  "select document_id from documents where collection_parent_path = $1 and collection_id = $2",
  &[&collection_parent_path, &collection_id]).unwrap().into_iter()
    .map(|row| row.get(0))
    .collect();

  let documents: Vec<Document> = document_ids.iter()
    .map(|document_id| get_document(transaction, user_id, collection_parent_path, collection_id, document_id).unwrap())
    .collect();

  documents
}

pub fn get_documents_from_collection_group(
  transaction: &mut Transaction,
  user_id: &UserId,
  collection_id: &str)
  -> Vec<Document>
{
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &None,
                                 collection_id, &None));
  }

  let document_id_rows: Vec<_> = transaction.query(
    "select collection_parent_path, document_id from documents where collection_id = $1",
    &[&collection_id]
  ).unwrap();

  let documents: Vec<Document> = document_id_rows.iter()
    .map(|document_id_row| get_document(transaction, user_id, document_id_row.get(0), collection_id, document_id_row.get(1)).unwrap())
    .collect();
  documents
}

pub fn get_matching_basic_subscription_ids(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
) -> Vec<String> {
  let document_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id from basic_subscriptions where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id]
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let collection_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id from basic_subscriptions where collection_parent_path=$1 and collection_id=$2 and document_id IS NULL",
    &[&collection_parent_path, &collection_id]
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let collection_group_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id from basic_subscriptions where collection_parent_path IS NULL and collection_id=$1 and document_id IS NULL",
    &[&collection_id]
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let all_matching_subscriptions: Vec<String> =
    document_subscriptions.into_iter()
      .chain(collection_subscriptions.into_iter())
      .chain(collection_group_subscriptions.into_iter())
      .collect();
  all_matching_subscriptions
}


pub fn subscribe_to_document(
  transaction: &mut Transaction,
  client_id: &str,
  user_id: &UserId,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str
) -> String
{
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::Get,
                                 &Some(collection_parent_path.to_string()),
                                 collection_id, &Some(document_id.to_string())));
  }

  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();
  transaction.execute("insert into basic_subscriptions values ($1, $2, $3, $4)",
                      &[&collection_parent_path, &collection_id, &document_id, &subscription_id]).unwrap();

  // Todo: trigger first subscription update?
  subscription_id
}

pub fn subscribe_to_collection(
  transaction: &mut Transaction,
  client_id: &str,
  user_id: &UserId,
  collection_parent_path: &str,
  collection_id: &str)
  -> String
{
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &Some(collection_parent_path.to_string()),
                                 collection_id, &None));
  }

  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();
  transaction.execute("insert into basic_subscriptions values ($1, $2, NULL, $3)",
                      &[&collection_parent_path, &collection_id, &subscription_id]).unwrap();

  // Todo: trigger first subscription update?
  subscription_id
}

pub fn subscribe_to_collection_group(
  transaction: &mut Transaction,
  client_id: &str,
  user_id: &UserId,
  collection_id: &str)
  -> String
{
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &None,
                                 collection_id, &None));
  }

  let subscription_id: String = Uuid::new_v4().as_simple().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();
  transaction.execute("insert into basic_subscriptions values (NULL, $1, NULL, $2)",
                      &[&collection_id, &subscription_id]).unwrap();

  // Todo: trigger first subscription update?
  subscription_id
}
