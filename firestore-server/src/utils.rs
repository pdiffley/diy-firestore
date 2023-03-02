use std::collections::{HashMap, HashSet};
use prost::Message;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use std::error::Error;
use std::env;
use std::fmt;
use bytes::{Bytes, BytesMut};

use itertools::Itertools;
use crate::basic_read::get_document;

use crate::protos::document_protos::Document;
use crate::protos::document_protos::FieldValue;
use crate::protos::document_protos::field_value::Value;
use crate::security_rules::UserId;
use crate::sql_types::{field_value, Unit};


pub fn get_document_from_row_id(transaction: &mut Transaction, user_id: &UserId, document_id_row: Row) -> Document {
  get_document(
    transaction,
    user_id,
    document_id_row.get("collection_parent_path"),
    document_id_row.get("collection_id"),
    document_id_row.get("document_id"))
    .unwrap()
}

pub fn field_value_proto_to_sql(field_value: &FieldValue) -> field_value {
  let mut sql_field_value = field_value {
    min:               None,
    null_value:        None,
    boolean_value:     None,
    integer_value:     None,
    double_value:      None,
    timestamp_nanos:   None,
    timestamp_seconds: None,
    string_value:      None,
    bytes_value:       None,
    reference_value:   None,
    max:               None
  };

  match field_value.value.clone().unwrap() {
    Value::NullValue(_) => sql_field_value.null_value = Some(Unit::NotNull),
    Value::BooleanValue(x) => sql_field_value.boolean_value = Some(x),
    Value::IntegerValue(x) => sql_field_value.integer_value = Some(x),
    Value::DoubleValue(x) => sql_field_value.double_value = Some(x),
    Value::TimestampValue(x) => { sql_field_value.timestamp_nanos = Some(x.nanos); sql_field_value.timestamp_seconds = Some(x.seconds); },
    Value::StringValue(x) => sql_field_value.string_value = Some(x),
    Value::BytesValue(x) => sql_field_value.bytes_value = Some(x),
    Value::ReferenceValue(x) => sql_field_value.reference_value = Some(x),
  }

  sql_field_value
}

pub fn null_sql_field_value() -> field_value {
  field_value {
    min:               None,
    null_value:        Some(Unit::NotNull),
    boolean_value:     None,
    integer_value:     None,
    double_value:      None,
    timestamp_nanos:   None,
    timestamp_seconds: None,
    string_value:      None,
    bytes_value:       None,
    reference_value:   None,
    max:               None
  }
}


