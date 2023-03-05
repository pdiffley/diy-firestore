### Real time updates

The ability to subscribe to changes in database state is a key feature of Firestore that distinguishes it over other databases. For any query, we we can make to the database, Firestore allows us to create a listener that receive's updates whenever the database changes in a way that would affect the queries results.

Supporting this feature introduces significant system design complications though. A naive approach would to have each listener poll our database for updates. This quickly becomes unscaleable because the polling would repeatedly send the same queries to the database to regardless of whether the query results have changed. Even at a small scale, this approach would hammer our database with requests, almost all of which would return no new information to the client. 

A better approach is to check for updates when we write to the database and send an update to any subscribed queries that are affected by a particular write. This raises another issue though. Assuming we have a large number of users each with their own set of subscriptions, we can’t just check every subscribed query to see if it is affected by the update every time we write to the database. 

When we write a document to the database, we need a way to efficiently select the set of subscriptions that are affected by the write. In short, we need to query the set of subscriptions to the database.

#### Subscriptions to documents, collections, and collection_groups

With that in mind, we will create a table "basic_subscriptions", where we will track subscriptions to documents, collections, and collection groups

```postgresql
CREATE TABLE basic_subscriptions (
  collection_parent_path      TEXT,
  collection_id               TEXT,
  document_id                 TEXT,
  subscription_id             TEXT
  PRIMARY KEY (subscription_id)
);
```

Each row represents a subscription and the relevant identifiers for it. A subscription to a document will specify all three document identifiers, collection_parent_path, collection_id, and document_id, a subscription to a collection will leave document_id null, and a subscription to a collection group will leave both colleciton_parent_path and document_id null. 

So that we can query these values efficiently, we'll create the index

```postgresql
CREATE INDEX basic_subscriptions_idx 
ON basic_subscriptions(collection_parent_path, collection_id, document_id);
```

Now we can write a function that will take a document id and return any subscriptions affected by that document. 

```rust
pub fn get_matching_basic_subscription_ids(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str) -> Vec<String>
{
  let document_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id
    from basic_subscriptions
    where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id],
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let collection_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id
    from basic_subscriptions
    where collection_parent_path=$1 and collection_id=$2 and document_id IS NULL",
    &[&collection_parent_path, &collection_id],
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let collection_group_subscriptions: Vec<String> = transaction.query(
    "SELECT subscription_id
    from basic_subscriptions
    where collection_parent_path IS NULL and collection_id=$1 and document_id IS NULL",
    &[&collection_id],
  ).unwrap().iter()
    .map(|x| x.get(0)).collect();

  let all_matching_subscriptions: Vec<String> =
    document_subscriptions.into_iter()
      .chain(collection_subscriptions.into_iter())
      .chain(collection_group_subscriptions.into_iter())
      .collect();
  all_matching_subscriptions
}
```

We will then add that function to our create and delete document methods. 

```rust
fn create_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  update_id: &str,
  document: &Document,
) {
  let mut encoded_document: Vec<u8> = vec![];
  document.encode(&mut encoded_document).unwrap();

  add_document_to_documents_table(transaction, collection_parent_path, collection_id, document_id, update_id, &encoded_document);

  let mut matching_subscriptions = vec![];
  matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());

  // Todo: send update to matching subscriptions
}
```

```rust
pub fn delete_document(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
) {
  if let Some(document) = get_document(transaction, collection_parent_path, collection_id, document_id) {
    delete_document_from_documents_table(transaction, collection_parent_path, collection_id, document_id);

    let mut matching_subscriptions = vec![];
    matching_subscriptions.extend(get_matching_basic_subscription_ids(transaction, collection_parent_path, collection_id, document_id).into_iter());

    // Todo: send update to matching subscriptions
  }
}
```

We will come back to sending the actual update later in section <update queue section>, but for now we know that whenever we write to the database we can efficiently get the list of subscriptions to pass the update to.

### Next up

We can now read and write documents to our database, and subscribe to document changes, but we can't query collections based on document data. Let's fix that.
