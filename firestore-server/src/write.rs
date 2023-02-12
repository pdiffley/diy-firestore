use std::collections::{HashMap, HashSet};
use prost::Message;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use std::error::Error;
use std::env;
use std::fmt;
use bytes::{Bytes, BytesMut};

use itertools::Itertools;
use crate::basic_read::{get_affected_basic_subscription_ids, get_document};
use crate::composite_query::{add_document_to_composite_query_tables, CompositeFieldGroup, delete_document_from_composite_query_tables, get_affected_composite_query_subscriptions};

use crate::protos::document_protos::Document;
use crate::protos::document_protos::FieldValue;
use crate::protos::document_protos::field_value::Value;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::simple_query::{add_document_to_simple_query_table, delete_document_from_simple_query_table, get_affected_simple_query_subscriptions};
use crate::sql_types::{SqlFieldValue};
use crate::update_queue::{add_update_to_queues};

fn create_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
) {
  let encoded_document = {
    let mut encoded_document: Vec<u8> = vec![];
    document.encode(&mut encoded_document).unwrap();
    encoded_document
  };
  transaction.execute(
    "insert into documents values ($1, $2, $3, $4)",
    &[&collection_parent_path, &collection_id, &document_id, &encoded_document]).unwrap();

  //Todo: Update composite subscription tables
  add_document_to_simple_query_table(transaction, collection_parent_path, collection_id, document_id, document);
  add_document_to_composite_query_tables(transaction, collection_parent_path, collection_id, document_id, document, composite_groups);

  let affected_subscriptions = {
    let mut affected_subscriptions = vec![];
    affected_subscriptions.extend(get_affected_basic_subscription_ids (transaction, collection_parent_path, collection_id, document_id).into_iter());
    affected_subscriptions.extend(get_affected_simple_query_subscriptions(transaction, collection_parent_path, collection_id, document).into_iter());
    affected_subscriptions.extend(get_affected_composite_query_subscriptions(transaction, document, composite_groups).into_iter());
    affected_subscriptions
  };

  add_update_to_queues(
    transaction,
    &affected_subscriptions,
    collection_parent_path,
    collection_id,
    document_id,
    &Some(encoded_document));
  // Todo: Ping client-server connection to trigger update (this would actually happen after the transaction)
}

pub fn delete_document(
  transaction: &mut Transaction,
  user_id: &UserId,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_groups: &[CompositeFieldGroup],
) {
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::Delete,
                                 &Some(collection_parent_path.to_owned()),
                                 collection_id, &Some(document_id.to_owned())));
  }

  if let Some(document) = get_document(transaction, user_id, collection_parent_path, collection_id, document_id) {
    transaction.execute(
      "delete from documents where collection_parent_path=$1, collection_id=$2, document_id=$3",
      &[&collection_parent_path, &collection_id, &document_id])
      .unwrap();

    delete_document_from_simple_query_table(transaction, collection_parent_path, collection_id, document_id);
    delete_document_from_composite_query_tables(transaction, collection_parent_path, collection_id, document_id, composite_groups);

    let affected_subscriptions = {
      let mut affected_subscriptions = vec![];
      affected_subscriptions.extend(get_affected_basic_subscription_ids (transaction, collection_parent_path, collection_id, document_id).into_iter());
      affected_subscriptions.extend(get_affected_simple_query_subscriptions(transaction, collection_parent_path, collection_id, &document).into_iter());
      affected_subscriptions.extend(get_affected_composite_query_subscriptions(transaction, &document, composite_groups).into_iter());
      affected_subscriptions
    };

    add_update_to_queues(
      transaction,
      &affected_subscriptions,
      collection_parent_path,
      collection_id,
      document_id,
      &None);
    // Todo: Ping client-server connection to trigger update (this would actually happen after the transaction)
  }
}

pub fn write_document(
  transaction: &mut Transaction,
  user_id: &UserId,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
)
{
  let collection_parent_path = document.id.clone().unwrap().collection_parent_path;
  let collection_id = document.id.clone().unwrap().collection_id;
  let document_id = document.id.clone().unwrap().document_id;

  let document_exists = transaction.query(
    "SELECT 1 FROM documents WHERE collection_parent_path=$1, collection_id=$2, document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id]
  ).unwrap().len() > 0;

  let operation: Operation;
  if document_exists {
    operation = Operation::Update;
  } else {
    operation = Operation::Create;
  }

  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &operation,
                                 &Some(collection_parent_path.to_owned()),
                                 &collection_id, &Some(document_id.to_owned())));
  }

  delete_document(transaction, &UserId::Admin, &collection_parent_path, &collection_id, &document_id, composite_groups);
  create_document(transaction, &collection_parent_path, &collection_id, &document_id, document, composite_groups);
}

