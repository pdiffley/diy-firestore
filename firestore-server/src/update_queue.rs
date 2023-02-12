use postgres::Transaction;

pub fn add_update_to_queues(
  transaction: &mut Transaction,
  affected_subscriptions: &[String],
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document_data: &Option<Vec<u8>>)
{
  for subscription_id in affected_subscriptions {
    transaction.execute(
      "insert into update_queues values ($1, $2, $3, $4, $5, $6)",
      &[&subscription_id, &collection_parent_path, &collection_id, &document_id, &document_data]).unwrap();
  }
}
