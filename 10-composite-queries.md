

# Composite Queries

Unlike simple queries, we will not support any possible query out of the box. If we tried support all possible composite queries, we would quickly run into a problem with combinatorial explosion. Instead, we will require the combination of fields that will be queried over to be specified in advance, and only the first field, the **primary field**, can be queried with inequality operators (<, <=, >, >=, !=). All other fields, the **secondary fields**, can only be filtered with the equality operator, =. 

Because we know the combinations of fields that need to be queried in advance, we can create tables and indexes built to support those queries specifically.

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

Since we have the field group specified in advance, we can make a lookup table specifically for it. We will assign the composite field group a unique id, "d8b8c614b73546daa1d85531dc412ef6", and create the lookup table composite_lookup_table_d8b8c614b73546daa1d85531dc412ef6

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

Each of these structs contains all of the information about a single field group and the tables affiliated with it.

We can now make a function that will query a specific lookup table. I am making the assumption that we have already identified what field group the query belongs to. Our composite queries need to be generated dynamically so I have started using a simple query builder to help with this. 

```rust
pub fn composite_query(transaction: &mut Transaction, parameters: &[QueryParameter], composite_group: &CompositeFieldGroup) -> Vec<Document> {
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
    .map(|row| get_document(transaction, row.get("collection_parent_path"),
                            row.get("collection_id"), row.get("document_id")).unwrap())
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

Here we construct our query string, directing the query to the correct query table based on the **group_id** and adding all of the constraints from the query parameters. We then execute the query to get the document ids and retrieve those documents from the documents table.

Next we will create functions to add or delete documents from a set of composite group's look up tables. These functions have to dynamically specify the query parameters for a particular table, so they get fairly complicated, but all they are doing is adding or deleting a row from the look up table for a composite field group.   

```rust
pub fn add_document_to_composite_query_tables(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_groups: &[CompositeFieldGroup],
)
{
  for composite_field_group in composite_groups {
    add_document_to_composite_query_table(transaction, collection_parent_path, collection_id, document_id, document, composite_field_group);
  }
}

fn add_document_to_composite_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  document: &Document,
  composite_field_group: &CompositeFieldGroup,
) {
  let (primary_value, secondary_values) = get_field_group_values(document, composite_field_group);

  let table_name = format!("\"{}\"", composite_field_group.lookup_table_name());
  let query_string = {
    let mut query = sql_query_builder::Insert::new()
      .insert_into(&table_name)
      .values("($1, $2, $3, $4");
    for i in 0..secondary_values.len() {
      query = query.values(&format!("${}", i + 5));
    }
    query = query.raw_after(sql_query_builder::InsertClause::Values, ")");
    query.as_string()
  };

  let mut args: Vec<&(dyn ToSql + Sync)> = vec![&collection_parent_path, &collection_id, &document_id, &primary_value];
  args.extend(secondary_values.iter().map(|x| x as &(dyn ToSql + Sync)));

  transaction.execute(&query_string, &args).unwrap();
}

fn get_field_group_values(
  document: &Document,
  composite_field_group: &CompositeFieldGroup,
) -> (field_value, Vec<field_value>) {
  let primary_value = field_value_proto_to_sql(document.fields.get(&composite_field_group.primary_field_name).unwrap());
  let mut secondary_values = vec![];
  for field_name in &composite_field_group.sorted_secondary_field_names {
    if let Some(value) = document.fields.get(field_name) {
      secondary_values.push(field_value_proto_to_sql(value));
    } else {
      secondary_values.push(null_sql_field_value());
    }
  }
  (primary_value, secondary_values)
}
```

```rust
pub fn delete_document_from_composite_query_tables(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_groups: &[CompositeFieldGroup],
)
{
  for composite_field_group in composite_groups {
    delete_document_from_composite_query_table(transaction, collection_parent_path, collection_id, document_id, composite_field_group)
  }
}

fn delete_document_from_composite_query_table(
  transaction: &mut Transaction,
  collection_parent_path: &str,
  collection_id: &str,
  document_id: &str,
  composite_field_group: &CompositeFieldGroup,
) {
  let query_string: String =
    format!("delete from \"{}\" where collection_parent_path=$1 and collection_id=$2 and document_id=$3",
            composite_field_group.lookup_table_name());
  transaction.execute(&query_string, &[&collection_parent_path, &collection_id, &document_id]).unwrap();
}
```



We will then call those functions from our create and delete functions. We are going to lean a little further into the deus ex here and assume that we have already identified the set of composite groups affected by a write.

```rust
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

  // Todo: send update to matching subscriptions
}
```

```rust
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

    // Todo: send update to matching subscriptions
  }
}
```






