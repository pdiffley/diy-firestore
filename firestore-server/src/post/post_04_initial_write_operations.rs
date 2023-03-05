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

use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::field_value;

fn create_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  document: &Document)
{
  let mut encoded_document: Vec<u8> = vec![];
  document.encode(&mut encoded_document).unwrap();

  add_document_to_documents_table(transaction, collection_parent_path, collection_id, document_id,
                                  update_id, &encoded_document);
}

pub fn add_document_to_documents_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  encoded_document: &[u8])
{
  transaction.execute(
    "insert into documents values ($1, $2, $3, $4, $5)",
    &[&collection_parent_path, &collection_id, &document_id, &encoded_document, &update_id]).unwrap();
}

// ===================================================================================================

pub fn delete_document_from_documents_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
) {
  transaction.execute(
    "delete from documents where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id])
    .unwrap();
}

fn delete_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
) {
  if let Some(document) = get_document(transaction, collection_parent_path, collection_id, document_id) {
    delete_document_from_documents_table(transaction, collection_parent_path, collection_id, document_id);
  }
}

// ===================================================================================================

fn write_document(
  transaction: &mut Transaction,
  mut document: Document,
)
{
  let collection_parent_path: String = document.id.clone().unwrap().collection_parent_path.clone();
  let collection_id: String = document.id.clone().unwrap().collection_id.clone();
  let document_id: String = document.id.clone().unwrap().document_id.clone();
  let update_id: String = Uuid::new_v4().as_simple().to_string();
  document.update_id = Some(update_id.clone());

  delete_document(transaction, &collection_parent_path, &collection_id, &document_id);
  create_document(transaction, &collection_parent_path, &collection_id, &document_id, &update_id, &document);
}

// ===================================================================================================
// ===================================================================================================
// ===================================================================================================


pub fn get_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str)
  -> Option<Document>
{
  let rows = transaction.query(
    "SELECT document_data
    from documents
    where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id],
  ).unwrap();

  if rows.len() == 0 {
    return None;
  }

  let encoded_document: Vec<u8> = rows[0].get(0);
  let document: Document = Document::decode(&encoded_document[..]).unwrap();
  Some(document)
}


pub fn get_documents(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str)
  -> Vec<Document>
{
  let document_ids: Vec<String> = transaction.query(
    "select document_id from documents where collection_parent_path = $1 and collection_id = $2",
    &[&collection_parent_path, &collection_id]).unwrap().into_iter()
    .map(|row| row.get(0))
    .collect();

  let documents: Vec<Document> = document_ids.iter()
    .map(|document_id|
      get_document(transaction, collection_parent_path, collection_id, document_id).unwrap())
    .collect();

  documents
}

pub fn get_documents_from_collection_group(
  transaction: &mut Transaction,
  collection_id: &str)
  -> Vec<Document>
{
  let document_id_rows: Vec<_> = transaction.query(
    "select collection_parent_path, document_id from documents where collection_id = $1",
    &[&collection_id],
  ).unwrap();

  let documents: Vec<Document> = document_id_rows.iter()
    .map(|document_id_row| get_document(transaction, document_id_row.get(0), collection_id, document_id_row.get(1)).unwrap())
    .collect();
  documents
}

// ===================================================================================================
// ===================================================================================================
// ===================================================================================================


