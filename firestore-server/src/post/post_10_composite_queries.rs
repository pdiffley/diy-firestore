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

use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::simple_query::{delete_document_from_simple_query_table, get_matching_simple_query_subscriptions};
use crate::sql_types::field_value;
use crate::utils::{field_value_proto_to_sql, null_sql_field_value};

// =================================================================================================
// =================================================================================================
// =================================================================================================

pub struct CompositeFieldGroup {
  pub group_id: String,
  pub collection_parent_path: Option<String>,
  pub collection_id: String,
  pub primary_field_name: String,
  pub sorted_secondary_field_names: Vec<String>,
}

impl CompositeFieldGroup {
  fn lookup_table_name(&self) -> String {
    format!("composite_lookup_table_{}", self.group_id)
  }
}

// =================================================================================================
// =================================================================================================
// =================================================================================================


pub fn composite_query(transaction: &mut Transaction, parameters: &[QueryParameter], composite_group: &CompositeFieldGroup) -> Vec<Document> {
  let query_string = {
    let mut query = sql_query_builder::Select::new()
      .select("collection_parent_path, collection_id, document_id")
      .from(&composite_group.lookup_table_name());
    for (i, parameter) in parameters.iter().enumerate() {
      let constraint = format!("{} {} ${}", parameter.field_name, parameter.operator, i + 1);
      query = query.where_clause(&constraint);
    }
    query.as_string()
  };

  let args: Vec<_> = parameters.iter().map(|p| &p.parameter as &(dyn ToSql + Sync)).collect();

  let documents: Vec<Document> = transaction.query(&query_string, &args[..])
    .unwrap()
    .into_iter()
    .map(|row| get_document(transaction, row.get("collection_parent_path"),
                            row.get("collection_id"), row.get("document_id")).unwrap())
    .collect();
  documents
}

pub struct QueryParameter {
  pub field_name: String,
  pub operator: String,
  pub parameter: field_value,
  pub is_primary: bool,
}


// =================================================================================================
// =================================================================================================
// =================================================================================================

pub fn add_document_to_composite_query_tables(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
)
{
  for composite_field_group in composite_groups {
    add_document_to_composite_query_table(transaction, collection_parent_path, collection_id, document_id, document, composite_field_group);
  }
}

fn add_document_to_composite_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_field_group: &CompositeFieldGroup,
) {
  let (primary_value, secondary_values) = get_field_group_values(document, composite_field_group);

  let table_name = format!("\"{}\"", composite_field_group.lookup_table_name());
  let query_string = {
    let mut query = sql_query_builder::Insert::new()
      .insert_into(&table_name)
      .values("($1, $2, $3, $4");
    for i in 0..secondary_values.len() {
      query = query.values(&format!("${}", i + 5));
    }
    query = query.raw_after(sql_query_builder::InsertClause::Values, ")");
    query.as_string()
  };

  let mut args: Vec<&(dyn ToSql + Sync)> = vec![&collection_parent_path, &collection_id, &document_id, &primary_value];
  args.extend(secondary_values.iter().map(|x| x as &(dyn ToSql + Sync)));

  transaction.execute(&query_string, &args).unwrap();
}

fn get_field_group_values(
  document: &Document,
  composite_field_group: &CompositeFieldGroup,
) -> (field_value, Vec<field_value>) {
  let primary_value = field_value_proto_to_sql(document.fields.get(&composite_field_group.primary_field_name).unwrap());
  let mut secondary_values = vec![];
  for field_name in &composite_field_group.sorted_secondary_field_names {
    if let Some(value) = document.fields.get(field_name) {
      secondary_values.push(field_value_proto_to_sql(value));
    } else {
      secondary_values.push(null_sql_field_value());
    }
  }
  (primary_value, secondary_values)
}

// =================================================================================================


pub fn delete_document_from_composite_query_tables(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_groups: &[CompositeFieldGroup],
)
{
  for composite_field_group in composite_groups {
    delete_document_from_composite_query_table(transaction, collection_parent_path, collection_id, document_id, composite_field_group)
  }
}

fn delete_document_from_composite_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_field_group: &CompositeFieldGroup,
) {
  let query_string: String =
    format!("delete from \"{}\" where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
            composite_field_group.lookup_table_name());
  transaction.execute(&query_string, &[&collection_parent_path, &collection_id, &document_id]).unwrap();
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

    // Todo: send update to matching subscriptions
  }
}


// =================================================================================================
// =================================================================================================
// =================================================================================================


// on write

// pub fn get_matching_composite_query_subscriptions(
//   transaction: &mut Transaction,
//   document: &Document,
//   composite_groups: &[CompositeFieldGroup],
// ) -> Vec<String> {
//   let mut matching_subscriptions: Vec<String> = vec![];
//   for composite_group in composite_groups {
//     matching_subscriptions.extend(get_matching_subscriptions_for_composite_group(transaction, document, composite_group).into_iter());
//   }
//   matching_subscriptions
// }

// fn get_matching_subscriptions_for_composite_group(
//   transaction: &mut Transaction,
//   document: &Document,
//   composite_group: &CompositeFieldGroup,
// ) -> Vec<String> {
//   let primary_field_name = &composite_group.primary_field_name;
//   let included_query_string = {
//     let mut included_query = sql_query_builder::Select::new()
//       .select("subscription_id")
//       .from(&format!("{}", composite_group.included_subscription_table_name()));
//
//     if document.fields.contains_key(primary_field_name) {
//       included_query = included_query
//         .where_clause(&format!("min_{0} <= $1", primary_field_name))
//         .where_clause(&format!("max_{0} >= $1", primary_field_name));
//     }
//
//     for (i, field_name) in composite_group.sorted_secondary_field_names.iter().enumerate() {
//       if document.fields.contains_key(field_name) {
//         included_query = included_query.where_clause(&format!("{0} = ${1}", field_name, i + 2));
//       }
//     }
//     included_query.as_string()
//   };
//
//   let excluded_query_string =
//     format!("select distinct subscription_id from {} where excluded_{} = $1",
//             composite_group.excluded_subscription_table_name(), primary_field_name);
//
//
//   let query_string = format!("({}) EXCEPT ({})",
//                              included_query_string, excluded_query_string);
//
//   let (primary_value, secondary_values) = get_field_group_values(document, composite_group);
//   let mut args: Vec<&(dyn ToSql + Sync)> = vec![&primary_value];
//   args.extend(secondary_values.iter().map(|x| x as &(dyn ToSql + Sync)));
//
//   let matching_subscription_ids = transaction.query(&query_string, &args).unwrap()
//     .into_iter()
//     .map(|x| x.get::<usize, String>(0))
//     .collect();
//
//   matching_subscription_ids
// }

// =================================================================================================
// =================================================================================================
// =================================================================================================
