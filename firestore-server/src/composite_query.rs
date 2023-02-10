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
  is_primary: bool,
}

pub struct CompositeFieldGroup {
  group_type: CompositeFieldGroupType,
  lookup_table_name: String,
  included_subscription_table_name: String,
  excluded_subscription_table_name: String,
  primary_field_name: String,
  sorted_secondary_field_names: Vec<String>,
}

pub enum CompositeFieldGroupType {
  Collection,
  CollectionGroup
}

fn composite_table_name_from_query_fields(collection_parent_path: &Option<String>, collection_id: &str, parameters: &[QueryParameter]) -> String{
  let mut query_fields: Vec<String> = vec![parameters.iter().filter(|x| x.is_primary).map(|x| x.field_name.clone()).next().unwrap()];
  let mut secondary_query_fields: Vec<String> = HashSet::<String>::from_iter(parameters.iter().filter(|x| !x.is_primary).map(|x| x.field_name.clone())).into_iter().collect();
  secondary_query_fields.sort();
  query_fields.extend(secondary_query_fields.into_iter());

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

pub fn composite_query(transaction: &mut Transaction, user_id: &UserId, collection_parent_path: &Option<String>, collection_id: &str, parameters: &[QueryParameter], composite_group: &CompositeFieldGroup) -> Vec<Document> {
  if let User(user_id) = user_id {
    assert!(operation_is_allowed(user_id, &Operation::List,
                                 &collection_parent_path,
                                 collection_id, &None));
  }

  let query_string = {
    let mut query = sql_query_builder::Select::new()
      .select("collection_parent_path, collection_id, document_id")
      .from(&composite_group.lookup_table_name);
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
  composite_field_group: &CompositeFieldGroup
) {
  let mut query_string: String =
    format!("delete from \"{}\" where collection_parent_path=$1, collection_id=$2, document_id=$3",
            composite_field_group.lookup_table_name);
  transaction.execute(&query_string, &[&collection_parent_path, &collection_id, &document_id]).unwrap();
}



pub fn get_affected_composite_query_subscriptions(
  transaction: &mut Transaction,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
) -> Vec<String> {
  let mut affected_subscriptions: Vec<String> = vec![];
  for composite_group in composite_groups {
    affected_subscriptions.extend(get_affected_subscriptions_for_composite_group(transaction, document, composite_group).into_iter());
  }
  affected_subscriptions
}

fn get_affected_subscriptions_for_composite_group(
  transaction: &mut Transaction,
  document: &Document,
  composite_group: &CompositeFieldGroup,
) -> Vec<String> {
  let primary_field_name = &composite_group.primary_field_name;
  let included_query_string = {
    let mut included_query = sql_query_builder::Select::new()
      .select("subscription_id")
      .from(&format!("\"{}\"", composite_group.included_subscription_table_name));

    if document.fields.contains_key(primary_field_name){
      included_query = included_query
        .where_clause(&format!("min_{0} <= $1", primary_field_name))
        .where_clause(&format!("max_{0} >= $1", primary_field_name));
    }

    for (i, field_name) in composite_group.sorted_secondary_field_names.iter().enumerate() {
      if document.fields.contains_key(field_name) {
        included_query = included_query.where_clause(&format!("{0} = ${1}", field_name, i + 1));
      }
    }
    included_query.as_string()
  };

  let excluded_query_string =
    format!("select distinct subscription_id from \"{}\" where excluded_{} != $1",
            composite_group.excluded_subscription_table_name, primary_field_name);

  let query_string = format!("{} EXCEPT {}",
                             included_query_string, excluded_query_string);

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
  for field_name in std::iter::once(&composite_field_group.primary_field_name).chain(composite_field_group.sorted_secondary_field_names.iter()) {
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
  sorted_parameters: &[QueryParameter],
  composite_group: &CompositeFieldGroup,
) {
  let subscription_id: String = Uuid::new_v4().to_string();
  transaction.execute("insert into client_subscriptions values ($1, $2)",
                      &[&subscription_id, &client_id]).unwrap();

  let mut primary_less_than_param = SqlFieldValue::min();
  let mut primary_greater_than_parameter = SqlFieldValue::max();
  let mut primary_excluded_parameters = vec![];
  let mut secondary_parameters = vec![];

  for parameter in sorted_parameters {
    if parameter.is_primary {
      match parameter.operator.as_str() {
        "<=" => primary_less_than_param = parameter.parameter.clone(),
        ">=" => primary_greater_than_parameter = parameter.parameter.clone(),
        "<" => {
          primary_less_than_param = parameter.parameter.clone();
          primary_excluded_parameters.push(parameter.parameter.clone());
        },
        ">" => {
          primary_greater_than_parameter = parameter.parameter.clone();
          primary_excluded_parameters.push(parameter.parameter.clone());
        },
        "=" => {
          primary_less_than_param = parameter.parameter.clone();
          primary_greater_than_parameter = parameter.parameter.clone();
        },
        "!=" => primary_excluded_parameters.push(parameter.parameter.clone()),
        _ => panic!("Invalid query argument provided")
      }
    } else {
      secondary_parameters.push(parameter.parameter.clone());
    }
  }

  let mut row_string = "($1, $2, $3".to_owned();
  for i in 0 .. secondary_parameters.len() {
    row_string.push_str(&format!(", ${}", i + 4));
  }
  row_string.push(')');
  let included_query_string = format!("insert into {} values {}", composite_group.included_subscription_table_name, row_string);
  let mut included_args: Vec<&(dyn ToSql + Sync)> = vec![&subscription_id, &primary_less_than_param, &primary_greater_than_parameter];
  for secondary_parameter in secondary_parameters.iter() {
    included_args.push(secondary_parameter);
  }

  transaction.execute(&included_query_string,
                      &included_args).unwrap();
  let excluded_query_string = format!("insert into {} values ($1, $2)", composite_group.excluded_subscription_table_name);
  for excluded_value in primary_excluded_parameters {
    transaction.execute(&excluded_query_string, &[&subscription_id, &excluded_value]).unwrap();
  }

  // Todo: trigger first subscription update?
}

