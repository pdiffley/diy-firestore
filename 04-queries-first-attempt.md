# Queries: first attempt

Because we are storing our schema-less document data in a single text column, we can’t directly query our document data. To get around this, we will create a lookup table that we will use to query our documents.

## Simple Queries

First, let’s look at how we will support simple queries on a single field. Firestore allows us to query documents in a collection based on a single field without specifying what fields we want to query in advance. Because we have no knowledge of what fields we will query in advance, we can’t structure the schema of our lookup table around specific queries. Instead, we need a generic lookup table that lets us query documents based on any field a user wants to search for at run time. The table below serves that purpose

| collection_parent_path | collection_id | document_id | field_name | field_value |
| ---------------------- | ------------- | ----------- | ---------- | ----------- |
| Text                   | Text          | Text        | Text       | FieldValue  |


Each row in our table has a unique identifier for a document (collection_parent_path, collection_path, doc_id), the name of a field within the document and that field’s value. Now, when we write a document to the database, we will also add all of its fields to our lookup table.

There is one odd thing about the schema of our table though. The field_value column has the type field_value. This is definitely not a native type supported by Postgres, so what is going on here?

### Creating a custom type

Any field in our document can have multiple data types which makes it difficult to store the field values in our table. Like with our protobuf, we need to create a type that can represent any of the subtypes that can be assigned to our field values. Conveniently, postgres has a "create type" function that will let us to just that. There are [several options](https://www.postgresql.org/docs/current/sql-createtype.html) for creating a type. We will create a composite type with type field_value with the following command

```postgresql
CREATE TYPE field_value AS (
  null_value        boolean,
  boolean_value     boolean,
  integer_value     int8,
  double_value      float8,
  timestamp_nanos   int8,
  timestamp_seconds int8,
  string_value      text,
  bytes_value       bytea,
  reference_value   text
);
```

Our field_value type has values that can be set for represent any of the datatypes that will be held in our database. We do not have the convenient "oneof" feature that protobuf has, but for a given field, we will set the unused types to null. Note that if our value represents a timestamp, both the timestamp_nanos and timestamp_seconds field will have a value.

Composite types in postgres behave similarly to rows in a table, and we can create an instance of our variable with similar syntax.

```postgres
cast((null, null, 5, null, null, null, null, null, null) as field_value)
```

Here we are creating a row type holding the integer value 5, then casting it so postgres will recognize it as our composite field_value type. 

### Composite Type Ordering

We have a problem though. Postgres does not know how to properly order our field_value type. Consider the following example. 

```
select * from ( values
	(cast((NULL, NULL, 2, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
	(cast((NULL, NULL, NULL, 1.0, NULL, NULL, NULL, NULL, NULL) as field_value))) 	as test_table(field_value)
order by field_value ASC;
```

Running this command which attempts to put two field values in ascending order results in the output

```
 field_value 
-------------
 (,,2,,,,,,)
 (,,,1,,,,,)
```

The default comparison operator for composite values evaluates fields based on the order they are listed. Because integer_value comes before float_value in our compsite type, Postgres considers our integer value 2 to be less than our float value 1. To query our field values correctly, we need to create custom comparison operators that tell Postgres how we want the values to be ordered. 

