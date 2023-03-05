use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;
use std::process::Command;

use bytes::{Bytes, BytesMut};
use itertools::Itertools;
use postgres::{Client, IsolationLevel, NoTls, Row, Transaction};
use postgres::types::{ToSql, Type};
use prost::Message;
use stopwatch::Stopwatch;
use uuid::Uuid;

use protos::document_protos::Document;
use protos::document_protos::DocumentId;
use protos::document_protos::field_value::Value;
use protos::document_protos::field_value::Value::IntegerValue;
use protos::document_protos::field_value::Value::StringValue;
use protos::document_protos::FieldValue;
use sql_types::field_value;

use crate::basic_read::{get_document, get_documents, get_documents_from_collection_group, subscribe_to_collection, subscribe_to_collection_group, subscribe_to_document};
use crate::composite_query::{composite_query, CompositeFieldGroup, CompositeFieldGroupType, QueryParameter, subscribe_to_composite_query};
use crate::security_rules::UserId;
use crate::simple_query::simple_query;
use crate::simple_query::subscribe_to_simple_query;
use crate::sql_types::Unit;
use crate::write::{delete_document, write_document};

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
mod post;

// create an alias for a Result that can contain any error
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
  println!("{:?}", (((i64::MAX - 1) as f64) as i64) == i64::MAX - 1);
  println!("{:?}", ((f64::MAX) as i64) as f64);
  println!("{:?}", ((f64::MAX) as i64) == i64::MAX);
  println!("{:?}", ((f64::NAN) as i64));
  println!("{:?}", ((f64::INFINITY) as i64));
  println!("{:?}", ((f64::MAX - 123.5) as i64) == i64::MAX);
  println!("{:?}", ((f64::MIN + 16.1) as i64) == i64::MIN);
  println!("{:?}", ((i64::MIN as f64) as i64) == i64::MIN);
  // mainish();
  Ok(())
}

fn mainish() {
  teardown_database();
  setup_database();

  let user: String = env::var("USER").unwrap();
  let connection_string = &format!("host=localhost user={} dbname=diy_firestore", user);
  let mut client: Client = Client::connect(connection_string, NoTls).unwrap();
  let mut transaction = client.build_transaction()
    .isolation_level(IsolationLevel::RepeatableRead)
    .start().unwrap();


  let user_doc_id_1 = DocumentId {
    collection_parent_path: "/".to_string(),
    collection_id: "users".to_string(),
    document_id: "mwEPmPPTrzoefwX".to_string(),
  };

  let client_id = Uuid::new_v4().as_simple().to_string();
  let user_id = UserId::User(Some(Uuid::new_v4().as_simple().to_string()));

  let document_subscription_id = subscribe_to_document(
    &mut transaction,
    &client_id,
    &user_id,
    &user_doc_id_1.collection_parent_path,
    &user_doc_id_1.collection_id,
    &user_doc_id_1.document_id,
  );

  let collection_subscription_id = subscribe_to_collection(
    &mut transaction,
    &client_id,
    &user_id,
    &user_doc_id_1.collection_parent_path,
    &user_doc_id_1.collection_id,
  );

  let collection_group_subscription_id = subscribe_to_collection_group(
    &mut transaction,
    &client_id,
    &user_id,
    "posts",
  );


  let mut age_field_value_25 = field_value::default();
  age_field_value_25.integer_value = Some(25);
  let simple_user_age_subscription_id = subscribe_to_simple_query(
    &mut transaction,
    &client_id,
    &user_id,
    &Some(user_doc_id_1.collection_parent_path.clone()),
    &user_doc_id_1.collection_id,
    "age",
    "=",
    &age_field_value_25,
  );

  let mut name_field_value = field_value::default();
  name_field_value.string_value = Some("Quinn".to_string());
  let simple_user_name_subscription_id = subscribe_to_simple_query(
    &mut transaction,
    &client_id,
    &user_id,
    &Some(user_doc_id_1.collection_parent_path.clone()),
    &user_doc_id_1.collection_id,
    "name",
    "=",
    &name_field_value,
  );

  let mut age_field_value_130 = field_value::default();
  age_field_value_130.integer_value = Some(130);
  let mut city_field_value = field_value::default();
  city_field_value.string_value = Some("New York".to_string());
  let mut name_field_value = field_value::default();
  name_field_value.string_value = Some("Avery".to_string());
  let mut zipcode_field_value = field_value::default();
  zipcode_field_value.string_value = Some("20390".to_string());

  let composite_field_group = CompositeFieldGroup {
    collection_parent_path: Some("/".to_string()),
    collection_id: "users".to_string(),
    group_id: "d8b8c614b73546daa1d85531dc412ef6".to_string(),
    primary_field_name: "age".to_string(),
    sorted_secondary_field_names: vec!["city".to_string(), "name".to_string(), "zipcode".to_string()],
  };
  let parameters = vec![
    QueryParameter {
      field_name: "age".to_string(),
      operator: ">=".to_owned(),
      parameter: age_field_value_25.clone(),
      is_primary: true,
    },
    QueryParameter {
      field_name: "age".to_string(),
      operator: "<".to_owned(),
      parameter: age_field_value_130.clone(),
      is_primary: true,
    },
    QueryParameter {
      field_name: "city".to_string(),
      operator: "=".to_owned(),
      parameter: city_field_value.clone(),
      is_primary: false,
    },
    QueryParameter {
      field_name: "name".to_string(),
      operator: "=".to_owned(),
      parameter: name_field_value.clone(),
      is_primary: false,
    },
    QueryParameter {
      field_name: "zipcode".to_string(),
      operator: "=".to_owned(),
      parameter: zipcode_field_value.clone(),
      is_primary: false,
    },
  ];

  let composite_subscription_id = subscribe_to_composite_query(
    &mut transaction,
    &client_id,
    &user_id,
    &parameters,
    &composite_field_group,
  );

  let mut user_1 = Document {
    id: Some(user_doc_id_1.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("John".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(25)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  let mut user_doc_id_2 = user_doc_id_1.clone();
  user_doc_id_2.document_id = "AAA".to_string();
  let user_2 = Document {
    id: Some(user_doc_id_2.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("Avery".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(24)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  let mut user_doc_id_3 = user_doc_id_1.clone();
  user_doc_id_3.document_id = "BBB".to_string();
  let user_3 = Document {
    id: Some(user_doc_id_3.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("Quinn".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(25)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  let mut user_doc_id_4 = user_doc_id_1.clone();
  user_doc_id_4.document_id = "CCC".to_string();
  let user_4 = Document {
    id: Some(user_doc_id_4.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("Avery".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(26)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  let mut user_doc_id_5 = user_doc_id_1.clone();
  user_doc_id_5.document_id = "DDD".to_string();
  let user_5 = Document {
    id: Some(user_doc_id_5.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("Avery".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(130)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  let mut user_doc_id_6 = user_doc_id_1.clone();
  user_doc_id_6.document_id = "EEE".to_string();
  let user_6 = Document {
    id: Some(user_doc_id_6.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("Avery".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(34)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  let post_id_doc_1 = DocumentId {
    collection_parent_path: "/users/AAA/".to_string(),
    collection_id: "posts".to_string(),
    document_id: "111".to_string(),
  };
  let post_1 = Document {
    id: Some(post_id_doc_1.clone()),
    fields: HashMap::from([
      ("message".to_string(), FieldValue { value: Some(StringValue("Hi".to_string())) })]),
    update_id: None,
  };

  let post_id_doc_2 = DocumentId {
    collection_parent_path: "/users/EEE/".to_string(),
    collection_id: "posts".to_string(),
    document_id: "222".to_string(),
  };
  let post_2 = Document {
    id: Some(post_id_doc_2.clone()),
    fields: HashMap::from([
      ("message".to_string(), FieldValue { value: Some(StringValue("Hi Back".to_string())) })]),
    update_id: None,
  };


  write_document(&mut transaction, &user_id, user_1.clone(), &vec![composite_field_group.clone()]);
  user_1.fields.insert("name".to_string(), FieldValue { value: Some(StringValue("Jack".to_string())) });
  user_1.fields.insert("age".to_string(), FieldValue { value: Some(IntegerValue(26)) });
  write_document(&mut transaction, &user_id, user_1.clone(), &vec![composite_field_group.clone()]);
  write_document(&mut transaction, &user_id, user_2.clone(), &vec![composite_field_group.clone()]);
  write_document(&mut transaction, &user_id, user_3.clone(), &vec![composite_field_group.clone()]);
  write_document(&mut transaction, &user_id, user_4.clone(), &vec![composite_field_group.clone()]);
  write_document(&mut transaction, &user_id, user_5.clone(), &vec![composite_field_group.clone()]);
  write_document(&mut transaction, &user_id, user_6.clone(), &vec![composite_field_group.clone()]);
  write_document(&mut transaction, &user_id, post_1.clone(), &vec![]);
  write_document(&mut transaction, &user_id, post_2.clone(), &vec![]);

  delete_document(&mut transaction, &user_id, &user_doc_id_4.collection_parent_path, &user_doc_id_4.collection_id, &user_doc_id_4.document_id, &vec![composite_field_group.clone()]);


  println!("document_subscription_id");
  println!("{}", document_subscription_id);
  get_subscription_updates(&mut transaction, &document_subscription_id);
  println!();
  println!("collection_subscription_id");
  println!("{}", collection_subscription_id);
  get_subscription_updates(&mut transaction, &collection_subscription_id);
  println!();
  println!("collection_group_subscription_id");
  println!("{}", collection_group_subscription_id);
  get_subscription_updates(&mut transaction, &collection_group_subscription_id);
  println!();
  println!("simple_user_age_subscription_id");
  println!("{}", simple_user_age_subscription_id);
  get_subscription_updates(&mut transaction, &simple_user_age_subscription_id);
  println!();
  println!("simple_user_name_subscription_id");
  println!("{}", simple_user_name_subscription_id);
  get_subscription_updates(&mut transaction, &simple_user_name_subscription_id);
  println!();
  println!("composite_subscription_id");
  println!("{}", composite_subscription_id);
  get_subscription_updates(&mut transaction, &composite_subscription_id);
  println!();


  println!("{:?}", get_document(&mut transaction, &user_id, "/", "users", "AAA"));
  println!();
  println!("{:?}", get_documents(&mut transaction, &user_id, "/", "users"));
  println!();
  println!("{:?}", get_documents_from_collection_group(&mut transaction, &user_id, "posts"));

  let mut age_field_value_30 = field_value::default();
  age_field_value_30.integer_value = Some(25);
  let simple_query_age_result = simple_query(
    &mut transaction,
    &user_id,
    &Some(user_doc_id_1.collection_parent_path.clone()),
    &user_doc_id_1.collection_id,
    "age",
    ">",
    &age_field_value_30,
  );
  for doc in simple_query_age_result {
    println!("{:?}", doc);
  }
  println!();

  let mut name_field_value_avery = field_value::default();
  name_field_value_avery.string_value = Some("Avery".to_string());
  let simple_query_name_result = simple_query(
    &mut transaction,
    &user_id,
    &Some(user_doc_id_1.collection_parent_path.clone()),
    &user_doc_id_1.collection_id,
    "name",
    "=",
    &name_field_value_avery,
  );
  for doc in simple_query_name_result {
    println!("{:?}", doc);
  }
  println!();


  let composite_query_result = composite_query(
    &mut transaction,
    &user_id,
    &parameters,
    &composite_field_group,
  );
  for doc in composite_query_result {
    println!("{:?}", doc);
  }
  println!();


  transaction.commit().unwrap();


  // Check status of update queues
  // run transaction
  // run failed transaction
  // Read doc
  // Query docs
  // Composite query
  // Confirm indexes are being used by entering sql queries in terminal
}


fn basic_subscription() {
  teardown_database();
  setup_database();

  // Connect to the local database diy_firestore.
  let user: String = env::var("USER").unwrap();
  let connection_string = &format!("host=localhost user={} dbname=diy_firestore", user);
  let mut client: Client = Client::connect(connection_string, NoTls).unwrap();
  let mut transaction = client.build_transaction()
    .isolation_level(IsolationLevel::RepeatableRead)
    .start().unwrap();

  let user_doc_id_1 = DocumentId {
    collection_parent_path: "/".to_string(),
    collection_id: "users".to_string(),
    document_id: "mwEPmPPTrzoefwX".to_string(),
  };

  let client_id = Uuid::new_v4().as_simple().to_string();
  let user_id = UserId::User(Some(Uuid::new_v4().as_simple().to_string()));

  let document_subscription_id = subscribe_to_document(
    &mut transaction,
    &client_id,
    &user_id,
    &user_doc_id_1.collection_parent_path,
    &user_doc_id_1.collection_id,
    &user_doc_id_1.document_id,
  );

  let mut user_1 = Document {
    id: Some(user_doc_id_1.clone()),
    fields: HashMap::from([
      ("name".to_string(), FieldValue { value: Some(StringValue("John".to_string())) }),
      ("age".to_string(), FieldValue { value: Some(IntegerValue(24)) }),
      ("city".to_string(), FieldValue { value: Some(StringValue("New York".to_string())) }),
      ("zipcode".to_string(), FieldValue { value: Some(StringValue("20390".to_string())) })]),
    update_id: None,
  };

  write_document(&mut transaction, &user_id, user_1.clone(), &vec![]);
  user_1.fields.insert("name".to_string(), FieldValue { value: Some(StringValue("Jack".to_string())) });
  write_document(&mut transaction, &user_id, user_1.clone(), &vec![]);


  println!("document_subscription_id");
  println!("{}", document_subscription_id);
  get_subscription_updates(&mut transaction, &document_subscription_id);
}


fn setup_database() {
  let home_dir: String =
    env::vars().filter(|&(ref k, _)|
      k == "HOME"
    ).next().unwrap().1;

  let create_composite_type_path = home_dir.clone() + "/diy-firestore/sql-setup/create_composite_type.sql";
  let create_tables_path = home_dir.clone() + "/diy-firestore/sql-setup/create_tables.sql";
  let create_sample_composite_tables_path = home_dir.clone() + "/diy-firestore/sql-setup/create_sample_composite_group_tables.sql";

  Command::new("createdb").arg("diy_firestore").output().unwrap();
  Command::new("psql").args(["-d", "diy_firestore", "-f", &create_composite_type_path]).output().unwrap();
  Command::new("psql").args(["-d", "diy_firestore", "-f", &create_tables_path]).output().unwrap();
  Command::new("psql").args(["-d", "diy_firestore", "-f", &create_sample_composite_tables_path]).output().unwrap();
}

fn teardown_database() {
  Command::new("dropdb").arg("diy_firestore").output().unwrap();
}

fn get_subscription_updates(transaction: &mut Transaction, subscription_id: &str) {
  let update_docs: Vec<(String, Option<Vec<u8>>)> = transaction.query(
    "SELECT document_id, document_data
    from update_queues
    where subscription_id = $1",
    &[&subscription_id],
  ).unwrap().into_iter().map(|x| (x.get(0), x.get(1))).collect();
  // println!("{:?}", update_docs);
  for encoded_document in update_docs.iter() {
    if let Some(encoded_document) = &encoded_document.1 {
      let document: Document = Document::decode(&encoded_document[..]).unwrap();
      println!("{:?}", document);
    } else {
      println!("{:?}", encoded_document.0);
    }
  }
}








