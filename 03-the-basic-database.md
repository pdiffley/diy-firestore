# DIY Firestore

## First step: The database

Before we jump into the more complex features of our database we need to set up an underlying datastore that will provide a foundation. Somewhat unintuitively, we will be using a SQL database as the underlying datastore for our NoSQL database. SQL’s gold standard ACID transactions make it easy to implement our broad feature set. I will specifically be using Postgresql version 15 and its client, psql, in my examples. 

<maybe cut>
Using a SQL database has some potential does have some implications for scalability however. One of Firestores selling points is that it highly scalable. Using a traditional SQL database, we are limited to vertically scaling our write capacity. Vertical scaling should be fine for most cases, but the idea that you might have to fully rearchitect your system at a future date because your database can't scale to your needs can feel troubling when you are starting a project (even if you know you are optimizing prematurely). That said, if we needed horizontal scalability, we could switch to a horizontally scaleable SQL database like Spanner (which now has a Postgres interface).

Now that we have chosen our backing datastore, let’s look at how we will use it to support our most basic features. 

## Storing our data Operations

We need a way to store schemaless documents in our SQL database. To do this we will define our first table, "Documents":

```postgresql
CREATE TABLE documents (
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  document_data               bytea,
);
```

As dicussed in <defining our requirements>, the columns collection_parent_path, collection_id, and document_id will together be the unique identifier for a document. The document_data column will hold a binary representation of the document itself.

We'll also add an index so that we can retrieve documents from the table based on their identifiers

```postgresql
create index collection_id_collection_parent_path_index 
on documents(collection_id, collection_parent_path, document_id, update_id);
```



To store our documents in the document_data column, we need a way to get a binary representation of our document. Protobuf's is a convenient way to do this. We will define our document proto to have a document id and a map of fields.

```protobuf
syntax = "proto3";
package protos.documents;

message Document {
  DocumentId id = 1;
  map<string, FieldValue> fields = 2;
}

message DocumentId {
    string collection_parent_path = 1;
    string collection_id = 2;
    string document_id = 3;
}
```

Protobuf messages have a schema though. To allow for multiple datatypes to be stored in our document, we define a type FieldValue that can hold all of the possible value our documents support. Protobuf's "oneof" feature conveniently allows us to indicate that any given FieldValue will only contain one of these types.

```protobuf
message FieldValue {
  oneof value {
    Unit null_value = 1;
    bool boolean_value = 2;
    int64 integer_value = 3;
    double double_value = 4;
    Timestamp timestamp_value = 5;
    string string_value = 6;
    bytes bytes_value = 7;
    string reference_value = 8;
  }
}

enum Unit {
  Exists = 0;
}

message Timestamp {
  int64 nanos = 1;
  int64 seconds = 2;
}
```

With this protobuf representation, we can serialize our documents to a binary format and store them in our document_data column. Note that using protobuf is also conventient because we can also use the same binary representation to pass documents between our client libraries and backend. 

A quick aside: JSON is another viable candidate for a serialized document format, but it does not distinguish between integer and floating point numeric types ("1" and "1.0" are considered identical in JSON). Maintaining that type information in JSON would be pretty inconvenient, so I went with protobuf instead. 

## Basic operations

<put write operation first>

Now that we know how we are storing our documents, let’s take a look at how we will implement our basic read/write capabilities. We need to be able to:

- Read a document from the database
- Get all documents in a collection and/or collection_group
- Write a document to the database

Our table already has an index on (collection_parent_path, collection_id, document_id), so given the full path to a document, we can easily read it from the database

```Rust
pub fn get_document(
  transaction: &mut Transaction, 
  user_id: &UserId, 
  collection_parent_path: &str, 
  collection_id: &str, 
  document_id: &str) 
  -> Option<Document> 
{
  let rows = transaction.query(
    "SELECT document_data 
    from documents 
    where collection_parent_path=$1, collection_id=$2, document_id=$3",
    &[&collection_parent_path, &collection_id, &document_id]
  ).unwrap();

  if rows.len() == 0 {
    return None
  }
  let encoded_document: Vec<u8> = rows[0].get(0);
  let document: Document = Document::decode(&encoded_document[..]).unwrap();
  Some(document)
}
```

To retrieve efficiently retrive all of the documents from a collection and a collection group we will create the index

```postgresql
create index collection_id_collection_parent_path_index 
on documents(collection_id, collection_parent_path);
```

Then we can write two functions to retrieve those documents

```rust
fn get_documents(
	sql_client: <insert client type>,
  collection_parent_path: &str, 
  collection_id: &str
) -> return type
{
  let query = "select (document_id, document_data) from documents 
  where collection_parent_path = $1, collection_id = $2"
  sql_client.execute(query, &[collection_parent_path, collection_id])
}
```

```rust
fn get_documents_from_collection_group(
	sql_client: <insert client type>,
  collection_id: &str
) -> return type
{
  let query = "select (document_id, document_data) from documents 
  where collection_id = $2"
  sql_client.execute(query, &[collection_parent_path, collection_id])
}
```



Our write funciton is going to end up doing most of the heavy lifting for our feature set, but right now it is pretty simple <switch to add and delete function to keep things more concise>

```rust
fn write_document(
  sql: <insert client type>, 
  collection_parent_path: &str, 
  collection_id: &str, 
  document_id: &str,
  document_data: Protobuf type
) -> return type
{
  // todo: write insert statement
  let query = "SELECT document_data from documents where 
  	collection_parent_path=$1, collection_id=$2, document_id=$3"
	sql_client.execute(
    query, 
    &[collection_parent_path, collection_id, document_id, serialized ]).unwrap();  
  // todo extract data from from and return
}
```

#### All together

We now have a database with the feature set of a basic document store, and the performance characteristics of SQL. Pretty cool right? 

Ok, maybe not so much. Let’s add the ability to subscribe to real time updates.



### Real time updates

The ability to subscribe to changes in database state is one of the key features of Firestore that would make you choose it over a regular database. For any query we we can make to the database, Firestore allows us to create a listener for that query receive updates whenever the database changes in a way that would affect the queries results.

Supporting this feature introduces significant system design complications though. A naive approach would be to keep a list of subscriptions and poll our database for updates. This quickly becomes unscaleable because we would have to poll our database with every query a user has subscribed to regardless of whether these query results have changed. Even at a small scale, this approach would hammer our database with requests, almost all of which would return no new information to the client listening. 

A better approach is to check for updates when we write to the database, and send an update to any subscribed queries that are affected by a particular write. This raises another issue though. Assuming we have a large number of users each with their own set of subscriptions, we can’t just check every subscription for changes every time we write to the database. 

When we write a document to the database, we need a way to efficiently select the set of subscriptions that are affected by the write. In short, we need to query our list of subscriptions.



#### Subscriptions to documents, collections, and collection_groups

Since we just introduced the ability to read individual documents and read all of the documents in a collection or collection group, we will create a that will allow us to update any subscriptions to those datasets

**basic_subscriptions**

| collection_parent_path | collection_id | document_id | subscription_id |
| ---------------------- | ------------- | ----------- | --------------- |
| text                   | text          | text        | text            |

Our "basic_subscriptions" table map a full document identifier to a subscription_id listening to that document. We will go into more detail on this later, but for now we will just know that whenever a user makes a subscription, we will assign that subscription an id, and record the subscription information in this table. If they subscribe to a document, all four fields will have values. If they subscribe to a collection, document_id will be null, and if the subscribe to a collection group, both document_id and collection_parent_path will be null.

So that we can query these values efficiently, we'll create the index
```postgresql
create index basic_subscriptions_idx (collection_parent_path, collection_id, document_id)
```

Then when we add or delete a document, we can get all of the affected subscriptions with the following queries

```postgresql
-- Get document subscriptions
select subscription_id from basic_subscriptions where collection_parent_path = <collection_parent_path>, collection_id = <collection_id>, document_id = <document_id>

-- Get collection subscriptions
select subscription_id from basic_subscriptions where collection_parent_path = <collection_parent_path>, collection_id = <collection_id>, document_id = null

-- Get collection group subscriptions
select subscription_id from basic_subscriptions where collection_parent_path = null, collection_id = <collection_id>, document_id = null
```

I will leave sending the update as a todo until section <building subscription service>, but we can proceed knowing that our database structure will allow us to efficiently identify what subscriptions need to be notified when a write is made.
