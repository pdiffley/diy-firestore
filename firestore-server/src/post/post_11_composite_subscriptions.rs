use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;

use bytes::{Bytes, BytesMut};
use itertools::Itertools;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use prost::Message;
use sql_query_builder;
use uuid::Uuid;
use crate::post::post_04_initial_write_operations::{add_document_to_documents_table, delete_document_from_documents_table, get_document};
use crate::post::post_05_basic_subscriptions::get_matching_basic_subscription_ids;
use crate::post::post_08_simple_queries::add_document_to_simple_query_table;
use crate::post::post_10_composite_queries::{add_document_to_composite_query_tables, CompositeFieldGroup, delete_document_from_composite_query_tables};

use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::simple_query::{delete_document_from_simple_query_table, get_matching_simple_query_subscriptions};
use crate::sql_types::field_value;
use crate::utils::{field_value_proto_to_sql, null_sql_field_value};

pub fn get_matching_composite_query_subscriptions(
  transaction: &mut Transaction,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
) -> Vec<String> {
  let mut matching_subscriptions: Vec<String> = vec![];
  // ...
  matching_subscriptions
}

fn create_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
) {
  let mut encoded_document: Vec<u8> = vec![];
  document.encode(&mut encoded_document).unwrap();

  add_document_to_documents_table(transaction, collection_parent_path, collection_id, document_id, update_id, &encoded_document);
  add_document_to_simple_query_table(transaction, collection_parent_path, collection_id, document_id, document);
  add_document_to_composite_query_tables(transaction, collection_parent_path, collection_id, document_id, document, composite_groups);

  let mut matching_subscriptions = vec![];
  matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());
  matching_subscriptions.extend(get_matching_simple_query_subscriptions(transaction, collection_parent_path, collection_id, document).into_iter());
  matching_subscriptions.extend(get_matching_composite_query_subscriptions(transaction, document, composite_groups).into_iter());

  // Todo: send update to matching subscriptions
}


pub fn delete_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_groups: &[CompositeFieldGroup],
) {
  if let Some(document) = get_document(transaction, collection_parent_path, collection_id, document_id) {
    delete_document_from_documents_table(transaction, collection_parent_path, collection_id, document_id);
    delete_document_from_simple_query_table(transaction, collection_parent_path, collection_id, document_id);
    delete_document_from_composite_query_tables(transaction, collection_parent_path, collection_id, document_id, composite_groups);

    let mut matching_subscriptions = vec![];
    matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());
    matching_subscriptions.extend(get_matching_simple_query_subscriptions(transaction, collection_parent_path, collection_id, &document).into_iter());
    matching_subscriptions.extend(get_matching_composite_query_subscriptions(transaction, &document, composite_groups).into_iter());

    // Todo: send update to matching subscriptions
  }
}

