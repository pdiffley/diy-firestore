use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;

use bytes::{Bytes, BytesMut};
use itertools::Itertools;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use prost::Message;

use crate::basic_read::get_document;
use crate::protos::document_protos::Document;
use crate::protos::document_protos::field_value::Value;
use crate::protos::document_protos::FieldValue;
use crate::security_rules::UserId;
use crate::sql_types::{field_value, Unit};

// pub fn get_document_from_row_id(transaction: &mut Transaction, user_id: &UserId, document_id_row: Row) -> Document {
//   get_document(
//     transaction,
//     user_id,
//     document_id_row.get("collection_parent_path"),
//     document_id_row.get("collection_id"),
//     document_id_row.get("document_id"))
//     .unwrap()
// }

pub fn field_value_proto_to_sql(field_value: &FieldValue) -> field_value {
  let mut sql_field_value = field_value {
    min: None,
    null_value: None,
    boolean_value: None,
    integer_value: None,
    double_value: None,
    timestamp_nanos: None,
    timestamp_seconds: None,
    string_value: None,
    bytes_value: None,
    reference_value: None,
    max: None,
  };

  match field_value.value.clone().unwrap() {
    Value::NullValue(_) => sql_field_value.null_value = Some(Unit::Exists),
    Value::BooleanValue(x) => sql_field_value.boolean_value = Some(x),
    Value::IntegerValue(x) => sql_field_value.integer_value = Some(x),
    Value::DoubleValue(x) => sql_field_value.double_value = Some(x),
    Value::TimestampValue(x) => {
      sql_field_value.timestamp_nanos = Some(x.nanos);
      sql_field_value.timestamp_seconds = Some(x.seconds);
    }
    Value::StringValue(x) => sql_field_value.string_value = Some(x),
    Value::BytesValue(x) => sql_field_value.bytes_value = Some(x),
    Value::ReferenceValue(x) => sql_field_value.reference_value = Some(x),
  }

  sql_field_value
}

pub fn null_sql_field_value() -> field_value {
  field_value {
    min: None,
    null_value: Some(Unit::Exists),
    boolean_value: None,
    integer_value: None,
    double_value: None,
    timestamp_nanos: None,
    timestamp_seconds: None,
    string_value: None,
    bytes_value: None,
    reference_value: None,
    max: None,
  }
}

//TODO: Fix this to avoid information loss
pub fn prepare_field_value_constraint(
  column_name: &str,
  operator: &str,
  arg_count: usize,
  value: &field_value)
  -> (String, Vec<field_value>)
{
  if value.integer_value.is_none() && value.double_value.is_none() {
    return no_op_field_value_constraint(column_name, operator, arg_count, value);
  }

  // if let Some(double_value) = value.double_value {
  //   if double_value != double_value.round() ||
  //     double_value > (i64::MAX as f64) ||
  //     double_value < (i64::MIN as f64)
  //   {
  //     return no_op_field_value_constraint(column_name, operator, arg_count, value);
  //   }
  // }

  // If the double value has no equivalent integer, return the default constraint
  if let Some(double_value) = value.double_value {
    if (double_value as i64) as f64 != double_value {
      return no_op_field_value_constraint(column_name, operator, arg_count, value);
    }
  }

  // If the integer value has no equivalent double, return the default constraint
  if let Some(integer_value) = value.integer_value {
    if (integer_value as f64) as i64 != integer_value {
      return return no_op_field_value_constraint(column_name, operator, arg_count, value);;
    }
  }


  return prepare_numeric_field_value_constraint(column_name, operator, arg_count, value);
}

fn no_op_field_value_constraint(
  column_name: &str,
  operator: &str,
  arg_count: usize,
  value: &field_value)
  -> (String, Vec<field_value>)
{
  return (format!("{} {} ${}", column_name, operator, arg_count), vec![value.clone()]);
}

fn prepare_numeric_field_value_constraint(
  column_name: &str,
  operator: &str,
  arg_count: usize,
  value: &field_value)
  -> (String, Vec<field_value>)
{
  let double_return_value = {
    let mut double_return_value = field_value::default();
    if let Some(double_value) = value.double_value {
      double_return_value.double_value = Some(double_value);
    } else {
      double_return_value.double_value = Some(value.integer_value.unwrap() as f64);
    }
    double_return_value
  };

  let integer_return_value = {
    let mut integer_return_value = field_value::default();
    if let Some(integer_value) = value.integer_value {
      integer_return_value.integer_value = Some(integer_value);
    } else {
      integer_return_value.integer_value = Some(value.double_value.unwrap() as i64);
    }
    integer_return_value
  };

  if operator == "<=" || operator == ">" {
    return (format!("{} {} ${}", column_name, operator, arg_count), vec![double_return_value]);
  }

  if operator == "<" || operator == ">=" {
    return (format!("{} {} ${}", column_name, operator, arg_count), vec![integer_return_value]);
  }

  let constraint: String;
  if operator == "=" {
    constraint = format!("({0} = ${1} or {0} = ${2})", column_name, arg_count, arg_count + 1);
  } else {
    constraint = format!("({0} != ${1} and {0} != ${2})", column_name, arg_count, arg_count + 1);
  }
  return (constraint, vec![double_return_value, integer_return_value]);
}