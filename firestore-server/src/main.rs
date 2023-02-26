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
mod transaction;

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

use std::process::Command;
use uuid::Uuid;

fn main() -> Result<()> {
  let subscription_id: String = Uuid::new_v4().to_string();
  println!("{}", subscription_id);
  // let output = Command::new("$HOME/diy-firestore/sql-setup/teardown-db.sh")
  //   .output().unwrap();

  /// let ecode = child.wait()
  ///                  .expect("failed to wait on child");
  ///
  /// assert!(ecode.success());


  // let output = Command::new("/Users/pd/diy-firestore/sql-setup/setup-db.sh")
  //   .output().unwrap();
  // let output = Command::new("$HOME/diy-firestore/sql-setup/teardown-db.sh")
  //   .output().unwrap();

  // println!("{:?}", output);

  // let doc = Document::default();
  // println!("{:?}", doc);
  // println!("Hello, world!");
  //
  // // Get the local username. The ? operator returns early if there is an error
  // let user: String = env::var("USER")?;
  //
  // // Connect to the local database diy_firestore.
  // let connection_string = &format!("host=localhost user={} dbname=diy_firestore", user);
  // let _client: Client = Client::connect(connection_string, NoTls)?;

  Ok(())
}

fn setup_database() {
  let home_dir: String =
    env::vars().filter(|&(ref k, _)|
      k == "HOME"
    ).next().unwrap().1;

  let create_composite_type_sql = home_dir.clone() + "/diy-firestore/sql-setup/create_composite_type.sql";
  let create_tables_sql = home_dir.clone() + "/diy-firestore/sql-setup/create_tables.sql";

  Command::new("createdb").arg("diy_firestore").output().unwrap();
  Command::new("psql").args(["-d", "diy_firestore", "-f", &create_composite_type_sql]).output().unwrap();
  Command::new("psql").args(["-d", "diy_firestore", "-f", &create_tables_sql]).output().unwrap();
}

fn teardown_database() {
  Command::new("dropdb").arg("diy_firestore").output().unwrap();
}






























