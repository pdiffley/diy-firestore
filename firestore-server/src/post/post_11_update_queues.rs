use postgres::Transaction;
use uuid::Uuid;
use crate::post::post_04_initial_write_operations::{add_document_to_documents_table, delete_document_from_documents_table, get_document};
use crate::post::post_05_basic_subscriptions::get_matching_basic_subscription_ids;
use crate::post::post_08_simple_queries::{add_document_to_simple_query_table, delete_document_from_simple_query_table};
use crate::post::post_09_simple_query_subscriptions::get_matching_simple_query_subscriptions;
use crate::post::post_10_composite_queries::{add_document_to_composite_query_tables, CompositeFieldGroup, delete_document_from_composite_query_tables};
use crate::post::post_11_composite_subscriptions::get_matching_composite_query_subscriptions;
use crate::protos::document_protos::Document;
use prost::Message;

pub fn write_change_to_update_queues(
  transaction: &mut Transaction,
  matching_subscriptions: &[String],
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  document_data: &Option<Vec<u8>>)
{
  for subscription_id in matching_subscriptions {
    transaction.execute(
      "delete from update_queues where subscription_id = $1 and collection_parent_path = $2 and collection_id = $3 and document_id = $4",
      &[&subscription_id, &collection_parent_path, &collection_id, &document_id],
    ).unwrap();
    transaction.execute(
      "insert into update_queues values ($1, $2, $3, $4, $5, $6)",
      &[&subscription_id, &collection_parent_path, &collection_id, &document_id, &document_data, &update_id]).unwrap();
  }
}

//=================================================================================================

fn create_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
) {
  let mut encoded_document: Vec<u8> = vec![];
  document.encode(&mut encoded_document).unwrap();

  add_document_to_documents_table(transaction, collection_parent_path, collection_id, document_id, update_id, &encoded_document);
  add_document_to_simple_query_table(transaction, collection_parent_path, collection_id, document_id, document);
  add_document_to_composite_query_tables(transaction, collection_parent_path, collection_id, document_id, document, composite_groups);

  let mut matching_subscriptions = vec![];
  matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());
  matching_subscriptions.extend(get_matching_simple_query_subscriptions(transaction, collection_parent_path, collection_id, document).into_iter());
  matching_subscriptions.extend(get_matching_composite_query_subscriptions(transaction, document, composite_groups).into_iter());

  write_change_to_update_queues(transaction, &matching_subscriptions, collection_parent_path, collection_id, document_id, update_id, &Some(encoded_document));
}


pub fn delete_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_groups: &[CompositeFieldGroup],
) {
  if let Some(document) = get_document(transaction, collection_parent_path, collection_id, document_id) {
    delete_document_from_documents_table(transaction, collection_parent_path, collection_id, document_id);
    delete_document_from_simple_query_table(transaction, collection_parent_path, collection_id, document_id);
    delete_document_from_composite_query_tables(transaction, collection_parent_path, collection_id, document_id, composite_groups);

    let mut matching_subscriptions = vec![];
    matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());
    matching_subscriptions.extend(get_matching_simple_query_subscriptions(transaction, collection_parent_path, collection_id, &document).into_iter());
    matching_subscriptions.extend(get_matching_composite_query_subscriptions(transaction, &document, composite_groups).into_iter());

    let update_id: String = Uuid::new_v4().as_simple().to_string();
    write_change_to_update_queues(transaction, &matching_subscriptions, collection_parent_path, collection_id, document_id, &update_id, &None);
  }
}
