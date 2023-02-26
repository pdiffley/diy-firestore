

# DIY Firestore

## Queries: Take Two

In the last two sections, we made a user define composite type, field_value, that can hold any of our documents' data types, and we create a custom operator class for that type so that postgres can correctly sort and index those values.

With our custom type in hand, we can provide support for querying our documents.

## Simple Queries

We'll start by creating the lookup table from <first query section> 

| collection_parent_path | collection_id | document_id | field_name | field_value |
| ---------------------- | ------------- | ----------- | ---------- | ----------- |
| Text                   | Text          | Text        | Text       | FieldValue  |

```
CREATE TABLE simple_query_lookup (
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  field_name                  text,
  field_value                 field_value
  primary key (collection_parent_path, collection_id, document_id, field_name)
);
```

We need to allow for two primary cases for simple queries, querying collections and query collection groups. 

```postgresql
create index simple_collection_query_idx on simple_query_lookup(collection_parent_path, collection_id, field_name, field_value);
```

Then we can efficiently query a collection for a particular field and operator <account for numeric lookup>

```
rust code
```

and we can also query a collection group (note the index we made above works for both cases)

We can support collection group queries with similar ease

```
create index simple_collection_group_query_idx on 
simple_query_lookup(collection_id, field_name, field_value);
```

```
rust code
```



### Updating the simple query table

Of course our lookup table won't do us any good if it is not in sync with the documents in our database.

We'll make a function to add a document's fields to the lookup table.

```rust
fn add_document_to_simple_query_table(
	sql: <insert client type>, 
  collection_parent_path: &str, 
  collection_id: &str, 
  document_id: &str,
  document_data: Protobuf type
)
{
	for field_key in document_data {
    // todo: insert field and value into simple_query_lookup table
  }
}
```

Then we'll add it to our add document function so that whenever we insert a new document to postgres, we also update the fields in 

```

fn write_document ...
{
	// call add_fields_to_simple_query_table(doc_info)  
}
```

And we'll do the same thing for when we delete a document

```
fn delete_document_from_simple_query_table(
	sql: <insert client type>, 
  collection_parent_path: &str, 
  collection_id: &str, 
  document_id: &str,
  document_data: Protobuf type
)
{
	for field_key in document_data {
    // todo: delete field and value into simple_query_lookup table
  }
}
```

```
fn delete document <update with above function>
```



Now whenever we modify the documents in our database the lookup table will be updated too. Because we are doing all these modifications in a single transaction, we don't have to worry about the tables being temporarily out of sync.


## Query Subscriptions

Now that we have added support for simple queries, we need to make sure that we can update any subscriptions to those queries. Our basic subscriptions table from <the basic database> does not take into account 

But now that we are querying data from our collections, we need a subscription table that is more precises
For example, if we had a collection of documents each holding user's name and the last time they opened our app:
```
{userName: string,
 openedAppAt: Timestamp}
```
We would not want the query
```
appOpenedTimestamps (where user: == braden).get()
```
To fire everytime any user opened the app on their phone.

To efficiently query the list of subscription id's to simple queries, we need a table that includes a subscriptions query information.

With that in mind we will create the table simple_query_subscriptions with the following schema

| collection_parent_path | collection_id | field | operator | field_value | subscription_id |
| ---------------------- | ------------- | ----- | -------- | ----------- | --------------- |
| text                   | text          | text  | text     | field_value | text            |

Here we are storing the field name, the operation, and query parameter from the where clause of our query in addition to collection and subscription ids. If a user created a listener for the query above, we would add the row (collection, collection, 'user', '==', fieldvalue('braden'), <subscription_id>). We'll create this table and a corresponding index with:)

```
CREATE TABLE simple_query_subscriptions (
  collection_parent_path      text,
  collection_id               text,
  field                       text,
  operator                    text,
  field_value                 field_value,
  subscription_id							subscription_id
);

create index simple_collection_group_query_idx on 
simple_query_subscriptions(collection_parent_path, collection_id, field, operator, field_value);
```

Now when we write a document to our database we can check each of the document's fields against our supported operators and identify any simple queries that are affected by the update

```
// Todo: function that check each field and operation for subscription updates
// Todo: add subscription check to add and delete functions
```

It's worth noting that from a Big(O) perspective this is acceptable, but we still have to do a **lot** of checks every time we write a document with even a moderate number of fields. Being able to make queries on any field, any time we want comes at a cost! While Firestore allows you to select fields to exclude from this indexing, there is a limit to how many fields you can exclude (and it is kind of a pain). Keep reading to the <customizations and improvements> section to see how we might improve on this. 
