use postgres::Row;
use postgres_types::{ToSql, FromSql};

#[derive(Debug, Clone, ToSql, FromSql)]
pub struct SqlFieldValue {
  pub null_value:        Option<bool>,
  pub boolean_value:     Option<bool>,
  pub integer_value:     Option<i64>,
  pub double_value:      Option<f64>,
  pub timestamp_nanos:   Option<i64>,
  pub timestamp_seconds: Option<i64>,
  pub string_value:      Option<String>,
  pub bytes_value:       Option<Vec<u8>>,
  pub reference_value:   Option<String>,
}

#[derive(Debug, ToSql, FromSql)]
enum Unit {
  NotNull,
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