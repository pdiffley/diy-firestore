

# Composite Queries

Our approach to supporting composite queries will be different for than simple queries. If we tried support all possible composite queries, we would quickly run into a problem with combinatorial explosion. However, because we know the combinations of fields that need to be queried in advance, we can create tables and indexes built to support those queries specifically, and avoid that problem.

Let an example collection of user data `users` 

<img src="/Users/pd/diy-firestore/images/collection-dark.png" alt="drawing" width="30"/> users

​		<img src="/Users/pd/diy-firestore/images/document-dark.png" alt="drawing" width="30"/> Morgan

​			`name : 'morgan'`
​			`age: 35`
​			`city: Memphis`
​			`zipcode: 38125`

​		<img src="/Users/pd/diy-firestore/images/document-dark.png" alt="drawing" width="30"/> Alex​

​			`name: alex`
​			`age: 20`
​			`city: Memphis`
​			`zipcode: 38125`

And let's say we want to make a query like 

```
query('users', 
      where('age', '>', 20), 
      where('name', '=', 'John')      
      where('city', '=', 'Memphis')     
      where('zipcode', '=', '38125')
```

To support this query, we would first specify the composite field group in a config file like so

```
{
  collectionPath: "users", 
  groupType: "COLLECTION", 
  fields: [{"age": "ASC"}, {"name": "ASC"}, {"city": "ASC"}, {"zipcode": "ASC"}]
}
```

Having this field group specified in advance allows us to make a lookup table specifically for this field group

"collection_composite_lookup_table/users/age/name/city/zipcode"

| collection_parent_path | collection_id | document_id | age        | name       | city       | zipcode    |
| ---------------------- | ------------- | ----------- | ---------- | ---------- | ---------- | ---------- |
| Text                   | Text          | Text        | FieldValue | FieldValue | FieldValue | FieldValue |

-> then we put the index on it and query

In practice, we would automate the creation and management of these tables, but we will just manually create them for this case study.



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

"collection_composite_subscription_table_inequalities/users/age/name/city/zipcode"

| subscription_id | min_age    | max_age    |
| --------------- | ---------- | ---------- |
| Text            | FieldValue | FieldValue |


"collection_composite_subscription_table_exclusions/users/age/name/city/zipcode"

| excluded_age | subscription_id |
| ------------ | --------------- |
| FieldValue   | Text            |


"collection_composite_subscription_table_equalities/users/age/name/city/zipcode"

| subscription_id | age        | name       | city       | zipcode    |
| --------------- | ---------- | ---------- | ---------- | ---------- |
| Text            | FieldValue | FieldValue | FieldValue | FieldValue |


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
