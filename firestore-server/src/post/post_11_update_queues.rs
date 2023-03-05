use postgres::Transaction;
use uuid::Uuid;

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
