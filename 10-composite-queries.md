

# Composite Queries

Our approach to supporting composite queries will be different for than simple queries. If we tried support all possible composite queries, we would quickly run into a problem with combinatorial explosion. However, because we know the combinations of fields that need to be queried in advance, we can create tables and indexes built to support those queries specifically, and avoid that problem.

Let's look an example collection of user data `users` 

<img src="/Users/pd/diy-firestore/images/collection-dark.png" alt="drawing" width="30"/> users

​		<img src="/Users/pd/diy-firestore/images/document-dark.png" alt="drawing" width="30"/> Morgan

​			`user_name : 'morgan'`
​			`age: 35`
​			`city: Memphis`
​			`zipcode: 38125`

​		<img src="/Users/pd/diy-firestore/images/document-dark.png" alt="drawing" width="30"/> Alex​

​			`user_name: alex`
​			`age: 20`
​			`city: Memphis`
​			`zipcode: 38125`

And let's say we want to make a query like

```typescript
query('users', 
      where('age', '>', 20), 
      where('user_name', '=', 'John')      
      where('city', '=', 'Memphis')     
      where('zipcode', '=', '38125'))
```

To support this query, we would first specify the composite field group in a config file like so

```
{
  collectionPath: "users", 
  groupType: "COLLECTION", 
  fields: [{"age"}, {"city"}, {"user_name"}, {"zipcode"}]
}
```

Having the field group specified in advance allows us to make a lookup table specifically for it. We will assign the composite field group a unique id, "d8b8c614b73546daa1d85531dc412ef6", and create the lookup table composite_lookup_table_d8b8c614b73546daa1d85531dc412ef6

```postgresql
CREATE TABLE composite_lookup_table_d8b8c614b73546daa1d85531dc412ef6 (
  collection_parent_path      TEXT,
  collection_id               TEXT,
  document_id                 TEXT,
  age                         field_value,
  city                        field_value,
  user_name                   field_value,
  zipcode                     field_value,
  PRIMARY KEY (collection_parent_path, collection_id, document_id)
);

CREATE INDEX composite_lookup_table_idx_d8b8c614b73546daa1d85531dc412ef6 
ON composite_lookup_table_d8b8c614b73546daa1d85531dc412ef6(age, city, user_name, zipcode);
```

In practice, we would automate the creation and management of these tables, but we will just manually create them for this case study.

In our code, we will assume that we have parsed our configuration into a set of CompositeFieldGroup structs.

```rust
pub struct CompositeFieldGroup {
  pub group_id: String,
  pub collection_parent_path: Option<String>,
  pub collection_id: String,
  pub primary_field_name: String,
  pub sorted_secondary_field_names: Vec<String>,
}

impl CompositeFieldGroup {
  fn lookup_table_name(&self) -> String {
    format!("composite_lookup_table_{}", self.group_id)
  }
}
```

Knowing the composite group that a query maps to, it is fairly easy to query these documents. Note: Our composite queryies need to be generated dynamically so I havestarted using a simple query builder to help with this. 

```rust
pub fn composite_query(
  transaction: &mut Transaction,
  parameters: &[QueryParameter],
  composite_group: &CompositeFieldGroup) -> Vec<Document> {
  let query_string = {
    let mut query = sql_query_builder::Select::new()
      .select("collection_parent_path, collection_id, document_id")
      .from(&composite_group.lookup_table_name());
    for (i, parameter) in parameters.iter().enumerate() {
      let constraint = format!("{} {} ${}", parameter.field_name, parameter.operator, i + 1);
      query = query.where_clause(&constraint);
    }
    query.as_string()
  };

  let args: Vec<_> = parameters.iter().map(|p| &p.parameter as &(dyn ToSql + Sync)).collect();

  let documents: Vec<Document> = transaction.query(&query_string, &args[..])
    .unwrap()
    .into_iter()
    .map(|row| get_document_from_row_id(transaction, row))
    .collect();
  documents
}

pub struct QueryParameter {
  pub field_name: String,
  pub operator: String,
  pub parameter: field_value,
  pub is_primary: bool,
}
```





=================================================================================
< leave out auto generation code>


Let's take a look at a code sample which will do that?


```
Block of code showing index creation in rust
```
We will actually combine fields from the same collection into one table and the put individual indexes on it

We can then write an arbitrary query on any set of indexed fields

=================================================================================



// on write

  - check if collection or group has a composite field group
  - if so write fields to table
  
  - need table of collection/group -> fields tracked



Going to do a little deus ex, and assume that we've already parsed our config file of collection groups to get a nicely formatted map.





Subscriptions

Btree index approach, table set up:

"collection_composite_subscription_table_included/users/age/name/city/zipcode"

| subscription_id | min_age    | max_age    | name       | city       | zipcode    |
| --------------- | ---------- | ---------- | ---------- | ---------- | ---------- |
| Text            | FieldValue | FieldValue | FieldValue | FieldValue | FieldValue |


"collection_composite_subscription_table_excluded/users/age/name/city/zipcode"

| subscription_id | excluded_age |
| --------------- | ------------ |
| Text            | FieldValue   |


For write

- map from collection and collection group to all affected composite groups

For query and subscription

- map from collection/field names combination to specific composite group


Better method with gin index

| subscription_id | min_age    | max_age    | excluded_ages | name       | city       | zipcode    |
| --------------- | ---------- | ---------- | ------------- | ---------- | ---------- | ---------- |
| Text            | FieldValue | FieldValue | FieldValue[]  | FieldValue | FieldValue | FieldValue |

Requires implementing gin index operator class for FieldValue. Will cover in next part of the series
which includes array and map data types

<tbd: write code for both multi table b-tree approach and single table gin approach or just gin approach?>



// Todo: use uuid for composite group id 

make table with

collection_parent_path, collection_id, primary_field_name, secondary_field_names[], composite_group_id

// Todo: update requirements chapter to ignore ASC/DESC right now
