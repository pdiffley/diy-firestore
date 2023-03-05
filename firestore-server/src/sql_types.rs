use postgres::Row;
use postgres_types::{FromSql, ToSql};

#[derive(Debug, Clone, ToSql, FromSql)]
pub struct field_value {
  pub min: Option<Unit>,
  pub null_value: Option<Unit>,
  pub boolean_value: Option<bool>,
  pub integer_value: Option<i64>,
  pub double_value: Option<f64>,
  pub timestamp_nanos: Option<i64>,
  pub timestamp_seconds: Option<i64>,
  pub string_value: Option<String>,
  pub bytes_value: Option<Vec<u8>>,
  pub reference_value: Option<String>,
  pub max: Option<Unit>,
}

impl field_value {
  pub(crate) fn default() -> field_value {
    field_value {
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
    }
  }

  pub(crate) fn max() -> field_value {
    let mut val = field_value::default();
    val.max = Some(Unit::Exists);
    val
  }

  pub(crate) fn min() -> field_value {
    let mut val = field_value::default();
    val.min = Some(Unit::Exists);
    val
  }
}

#[derive(Debug, Clone, ToSql, FromSql)]
pub enum Unit {
  Exists,
}

#[derive(Debug, ToSql, FromSql)]
pub struct SqlDocumentId {
  pub collection_parent_path: String,
  pub collection_id: String,
  pub document_id: String,
}

impl SqlDocumentId {
  pub fn from_row(row: Row) -> SqlDocumentId {
    SqlDocumentId {
      collection_parent_path: row.get("collection_parent_path"),
      collection_id: row.get("collection_id"),
      document_id: row.get("document_id"),
    }
  }
}