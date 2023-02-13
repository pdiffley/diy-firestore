pub mod protos;
mod sql_types;
mod basic_read;
mod write;
mod simple_query;
mod composite_query;
mod utils;
mod security_rules;
mod update_queue;
mod client_connection_endpoint;

use std::collections::{HashMap, HashSet};
use prost::Message;
use postgres::{Client, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use std::error::Error;
use std::env;
use std::fmt;
use bytes::{Bytes, BytesMut};

use itertools::Itertools;

use protos::document_protos::Document;
use protos::document_protos::FieldValue;
use protos::document_protos::field_value::Value;
use sql_types::{SqlFieldValue};

// create an alias for a Result that can contain any error
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
  let doc = Document::default();
  println!("{:?}", doc);
  println!("Hello, world!");

  // Get the local username. The ? operator returns early if there is an error
  let user: String = env::var("USER")?;

  // Connect to the local database diy_firestore.
  let connection_string = &format!("host=localhost user={} dbname=diy_firestore", user);
  let _client: Client = Client::connect(connection_string, NoTls)?;

  Ok(())
}































