use std::collections::{HashMap, HashSet};
use prost::Message;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use std::error::Error;
use std::env;
use std::fmt;
use bytes::{Bytes, BytesMut};
use sql_query_builder;
use itertools::Itertools;
use uuid::Uuid;

use crate::protos::document_protos::Document;
use crate::protos::document_protos::FieldValue;
use crate::protos::document_protos::field_value::Value;
use crate::security_rules::{Operation, operation_is_allowed, UserId};
use crate::security_rules::UserId::User;
use crate::sql_types::{SqlFieldValue};
use crate::utils::{field_value_proto_to_sql, get_document_from_row_id, null_sql_field_value};


pub struct QueryParameter {
  field_name: String,
  operator: String,
  parameter: SqlFieldValue,
}

pub struct CompositeFieldGroup {
  group_type: CompositeFieldGroupType,
  lookup_table_name: String,
  inequality_subscription_table_name: String,
  exclusion_subscription_table_name: String,
  equality_subscription_table_name: String,
  primary_field_name: String,
  secondary_sorted_field_names: Vec<String>,
}

pub enum CompositeFieldGroupType {
  Collection,
  CollectionGroup
}

fn composite_table_name_from_query_fields(collection_parent_path: &Option<String>, collection_id: &str, parameters: &[QueryParameter]) -> String{
  let query_fields: HashSet<String> = HashSet::from_iter(parameters.iter().map(|x| x.field_name.clone()));
  let mut query_fields: Vec<String> = query_fields.into_iter().collect();
  query_fields.sort();

  composite_table_name(collection_parent_path, collection_id, &query_fields)
}

fn composite_table_name(collection_parent_path: &Option<String>, collection_id: &str, sorted_field_names: &[String]) -> String {
  format!("{}/{}", composite_table_collection_path(collection_parent_path, collection_id), composite_table_field_path(sorted_field_names))
}

fn composite_table_collection_path(collection_parent_path: &Option<String>, collection_id: &str) -> String {
  return if let Some(collection_parent_path) = collection_parent_path {
    format!("collection_composite_lookup_table/{}/{}", collection_parent_path, collection_id)
  } else {
    format!("collection_group_composite_lookup_table/{}", collection_id)
  }
}

fn composite_table_field_path(sorted_field_names: &[String]) -> String {
  sorted_field_names.join("/")
}

pub fn composite_query(transaction: &mut Transaction, user_id: &UserId, collection_parent_path: &Option<String>, collection_id: &str, parameters: &[QueryParameter]) -> Vec<Document> {
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &collection_parent_path,
                                 collection_id, &None));
  }

  let table_name: String = format!("\"{}\"", composite_table_name_from_query_fields(collection_parent_path, collection_id, parameters));

  let query_string = {
    let mut query = sql_query_builder::Select::new()
      .select("collection_parent_path, collection_id, document_id")
      .from(&table_name);
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
    .map(|row| get_document_from_row_id(transaction, user_id,row))
    .collect();
  documents
}

pub fn add_document_to_composite_query_tables(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_groups: &HashMap<String, Vec<CompositeFieldGroup>>,
)
{
  let composite_table_collection_id = composite_table_collection_path(&Some(collection_parent_path.to_string()), collection_id);
  let composite_table_collection_group_id = composite_table_collection_path(&None, collection_id);

  if let Some(affected_collection_tables) = composite_groups.get(&composite_table_collection_id) {
    for composite_field_group in affected_collection_tables {
      add_document_to_composite_query_table(transaction, collection_parent_path, collection_id, document_id, document, composite_field_group)
    }
  }

  if let Some(affected_collection_group_tables) = composite_groups.get(&composite_table_collection_group_id) {
    for composite_field_group in affected_collection_group_tables {
      add_document_to_composite_query_table(transaction, collection_parent_path, collection_id, document_id, document, composite_field_group)
    }
  }
}

fn add_document_to_composite_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_field_group: &CompositeFieldGroup
) {
  let document_values = get_field_group_values(document, composite_field_group);

  let table_name = format!("\"{}\"", composite_field_group.lookup_table_name);
  let query_string = {
    let mut query = sql_query_builder::Insert::new()
      .insert_into(&table_name)
      .values("($1, $2, $3");
    for i in 0 .. document_values.len() {
      query = query.values(&format!("{}", i + 4));
    }
    query = query.raw_after(sql_query_builder::InsertClause::Values, ")");
    query.as_string()
  };

  let mut args: Vec<&(dyn ToSql + Sync)> = vec![&collection_parent_path, &collection_id, &document_id];
  args.extend(document_values.iter().map(|x| x as &(dyn ToSql + Sync)));

  transaction.execute(&query_string, &args).unwrap();
}

pub fn delete_document_from_composite_query_tables(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_groups: &HashMap<String, Vec<CompositeFieldGroup>>,
)
{
  let composite_table_collection_id = composite_table_collection_path(&Some(collection_parent_path.to_string()), collection_id);
  let composite_table_collection_group_id = composite_table_collection_path(&None, collection_id);

  if let Some(affected_collection_tables) = composite_groups.get(&composite_table_collection_id) {
    for composite_field_group in affected_collection_tables {
      delete_document_from_composite_query_table(transaction, collection_parent_path, collection_id, document_id, composite_field_group)
    }
  }

  if let Some(affected_collection_group_tables) = composite_groups.get(&composite_table_collection_group_id) {
    for composite_field_group in affected_collection_group_tables {
      delete_document_from_composite_query_table(transaction, collection_parent_path, collection_id, document_id, composite_field_group)
    }
  }
}

fn delete_document_from_composite_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_field_group: &CompositeFieldGroup
) {
  let mut query_string: String =
    format!("delete from \"{}\" where collection_parent_path=$1, collection_id=$2, document_id=$3",
            composite_field_group.lookup_table_name);
  transaction.execute(&query_string, &[&collection_parent_path, &collection_id, &document_id]).unwrap();
}



pub fn get_affected_composite_query_subscriptions(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document: &Document,
  composite_groups: &HashMap<String, Vec<CompositeFieldGroup>>,
) -> Vec<String> {

  let composite_table_collection_id = composite_table_collection_path(&Some(collection_parent_path.to_string()), collection_id);
  let composite_table_collection_group_id = composite_table_collection_path(&None, collection_id);

  let mut affected_subscriptions: Vec<String> = vec![];
  if let Some(affected_collection_tables) = composite_groups.get(&composite_table_collection_id) {
    for composite_group in affected_collection_tables {
      affected_subscriptions.extend(get_affected_subscriptions_for_composite_group(transaction, document, composite_group).into_iter());
    }
  }

  if let Some(affected_collection_group_tables) = composite_groups.get(&composite_table_collection_group_id) {
    for composite_group in affected_collection_group_tables {
      affected_subscriptions.extend(get_affected_subscriptions_for_composite_group(transaction, document, composite_group).into_iter());
    }
  }

  affected_subscriptions
}

fn get_affected_subscriptions_for_composite_group(
  transaction: &mut Transaction,
  document: &Document,
  composite_group: &CompositeFieldGroup,
) -> Vec<String> {
  let primary_field_name = &composite_group.primary_field_name;

  let inequality_query_string =
    format!("select subscription_id from \"{}\" where min_{1} <= $1, max_{1} >= $1",
            composite_group.inequality_subscription_table_name, primary_field_name);

  let exclusion_query_string =
    format!("select distinct subscription_id from \"{}\" where excluded_{} != $1",
            composite_group.exclusion_subscription_table_name, primary_field_name);

  let equality_query_string = {
    let mut equality_query = sql_query_builder::Select::new()
      .select("subscription_id")
      .from(&format!("\"{}\"", composite_group.equality_subscription_table_name));
    
    for (i, field_name) in composite_group.sorted_field_names.iter().enumerate() {
      //TODO: Check if document has field
      equality_query = equality_query.where_clause(&format!("({0} = ${1} or {0} is null)", field_name, i));
    }
    equality_query.as_string()
  };

  let query_string = format!("({} INTERSECT {}) EXCEPT {}",
                             inequality_query_string, equality_query_string, exclusion_query_string);

  let document_values = get_field_group_values(document, composite_group);
  let args: Vec<&(dyn ToSql + Sync)> = document_values.iter().map(|x| x as &(dyn ToSql + Sync)).collect();

  let affected_subscription_ids = transaction.query(&query_string, &args).unwrap()
    .into_iter()
    .map(|x| x.get::<usize, String>(0))
    .collect();

  affected_subscription_ids
}

fn get_field_group_values(
  document: &Document,
  composite_field_group: &CompositeFieldGroup,
) -> Vec<SqlFieldValue> {
  let mut document_values = vec![];
  for field_name in &composite_field_group.sorted_field_names {
    if let Some(value) = document.fields.get(field_name) {
      document_values.push(field_value_proto_to_sql(value));
    } else {
      document_values.push(null_sql_field_value());
    }
  }
  document_values
}

//Todo: Note constraint that all fields in a group need to be included in a query (simplify table selection)
pub fn subscribe_to_composite_query(
  transaction: &mut Transaction,
  client_id: &str,
  collection_parent_path: &Option<String>,
  collection_id: &str,
  parameters: &[QueryParameter],
  composite_groups: &HashMap<String, CompositeFieldGroup>,
) {
  let subscription_id: String = Uuid::new_v4().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();


  let table_id: String = format!("\"{}\"", composite_table_name_from_query_fields(collection_parent_path, collection_id, parameters));
  let composite_group = composite_groups[table_id];


  pub struct CompositeFieldGroup {
    group_type: CompositeFieldGroupType,
    lookup_table_name: String,
    inequality_subscription_table_name: String,
    exclusion_subscription_table_name: String,
    equality_subscription_table_name: String,
    sorted_field_names: Vec<String>,
  }


  // let query_fields: HashSet<String> = HashSet::from_iter(parameters.iter().map(|x| x.field_name.clone()));
  // let mut query_fields: Vec<String> = query_fields.into_iter().collect();
  // query_fields.sort();
  //
  // let table_name: String = format!("\"{}\"", composite_table_name(collection_parent_path, collection_id, &query_fields));
  //
  //
  // let collection_parent_path_string: String;
  // if let Some(collection_parent_path) = collection_parent_path {
  //   collection_parent_path_string = collection_parent_path.clone();
  // } else {
  //   collection_parent_path_string = "NULL".to_owned();
  // }
  //
  // transaction.execute("insert into simple_query_subscriptions values ($1, $2, $3, $4, $5, $6)",
  //                     &[&collection_parent_path_string, &collection_id, &field_name, &field_operator, &field_value, &subscription_id]).unwrap();
  //
  // // Todo: trigger first subscription update?
}

