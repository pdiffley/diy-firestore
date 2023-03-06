use std::thread::sleep;
use std::time::{Duration, SystemTime};

use postgres::Client;

const LONG_POLL_TIME_SECONDS: u64 = 20;

pub fn listen_for_update(sql_client: &mut Client, user_client_id: &str) {
  record_client_ping(sql_client, user_client_id);

  // Todo: on message queue notification:
  //    close request with message that client should retrieve updates

  if client_is_out_of_date(sql_client, user_client_id) {
    // Todo: close request with message that client should retrieve updates
  }

  // would be replaced with an async version of this
  sleep(Duration::new(LONG_POLL_TIME_SECONDS, 0));
  // Todo: close request with message that client is up to date
}

pub fn record_client_ping(sql_client: &mut Client, user_client_id: &str) {
  let now = SystemTime::now();
  sql_client.execute(
    "insert into client_ping_times values ($1, $2)",
    &[&user_client_id, &now]).unwrap();
}

pub fn client_is_out_of_date(sql_client: &mut Client, user_client_id: &str) -> bool {
  return sql_client.query(
    "SELECT 1 FROM client_subscriptions C JOIN update_queues U
     ON C.subscription_id = U.subscription_id 
     WHERE C.client_id = $1 
     LIMIT 1",
    &[&user_client_id])
    .unwrap().len() == 0;
}

pub struct UpdateValue {
  subscription_id: String,
  collection_parent_path: String,
  collection_id: String,
  document_id: String,
  document_data: Option<Vec<u8>>,
  update_id: String,
}

pub fn get_updates(sql_client: &mut Client, user_client_id: &str) -> Vec<UpdateValue> {
  sql_client.query(
    "SELECT subscription_id, collection_parent_path, collection_id, document_id, document_data, update_id
     FROM client_subscriptions C JOIN update_queues U 
     ON C.subscription_id = U.subscription_id 
     WHERE C.client_id = $1 
     LIMIT 1",
    &[&user_client_id])
    .unwrap().into_iter()
    .map(|row| UpdateValue {
      subscription_id: row.get(0),
      collection_parent_path: row.get(1),
      collection_id: row.get(2),
      document_id: row.get(3),
      document_data: row.get(4),
      update_id: row.get(5),
    })
    .collect()
}

//Todo: needs verification and index
pub fn confirm_updates(sql_client: &mut Client, user_client_id: &str, update_ids: &[String]) {
  sql_client.execute(
    "delete FROM update_queues U USING client_subscriptions C 
     where U.subscription_id = C.subscription_id and C.client_id = $1 and U.update_id IN $2",
    &[&user_client_id, &update_ids]).unwrap();
}




