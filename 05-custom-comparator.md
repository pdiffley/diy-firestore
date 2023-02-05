# DIY Firestore

## Create custom comparison operators

In the previous section, we created a custom composite type "field_value".

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

Postgres's default comparison operators do not order our field_value type correctly, so we need to create a set of custom comparison operators that Postgres can use to sort and build indexes for our values.

Fortunately, Postgres has a built in way for us to implement custom comparison operators. Postgres also has some specific and fine grained language relating to operators, their underlying functions, and indexes relate to each other, and that terminology is not always intuitive. Let's go over that now so we all know what we're talking about.

### Terminology

**Operator**

An operator is the symbol representing some operation (eg. ">", "<" "="). It is the symbol you put in a "where" clause to limit the scope of a query. 

Postgres allows define a new operator and associate it with a function that will be called when the operator is used. Operators can be overloaded for different types, so we can use standard operator symbols for our custom type. We can use the "<" symbol to perform a "less than" operation for example.

**Strategies and support functions**

Because users can define their own operators, index methods (eg. btree, hashmap, gin) are not implemented using hardcoded operators. Instead an index method has a numbered list of "strategies" and "support functions" representing operations a data type needs to support in order to be compatible with the index method. For example the [btree index method](https://www.postgresql.org/docs/current/btree-behavior.html) requires five strategies, 1: "less than", 2: "less than or equal", 3: "equal", 4: "greater than or equal", and 5: "greater than", but the operator symbols that are used to implement those strategies is not hard coded.

To tell the database what operators should be used to create an index for a particular type, we create an operator class.

**Operator Class**

An [operator class](https://www.postgresql.org/docs/14/sql-createopclass.html) is a collection of operators and functions that are mapped strategy numbers for an index. It tells the database what functions and operators should be used to create an index for a given type. 

To create a btree index for our field_value type, we need to create an operator class with a command like

```postgresql
create operator class field_value_ops
    default for type field_value using btree as
        operator        1       < ,
        operator        2       <= ,
        operator        3       = ,
        operator        4       >= ,
        operator        5       > ,
        function        1       field_value_cmp(field_value, field_value);
```

This command creates the operator class "field_value_ops" and tells the database to use the operator class as the default set of comparison functions for field_values when indexing with a btree. The command assigns operators, "<", "<=", "=", ">=", and ">", to the btree's corresponding strategy numbers, and assigns the function "field_value_cmp" to the btree's corresponding support function number. Of course before we can make this operator class, we need to implement these operators and functions for our field_value type.

Before we implement out operators, I'd like point out again that because of the way postgres maps operators to indexing "strategies", we did not have to use the operator symbols in the command above. Even though the strategy number 1 expects a "less than" operation we did not need to assign that strategy an operator with the "<" symbol. As long as the operator we assign the to strategy 1 performs a "less than" comparison for two field values, the index is happy.

So if you were feeling particularly excited one day and wanted to pass that enthusiam along to your database, you could assign the "less than or equal" strategy the operator "< =D". Fun right? Who ever said database engineers don't know how to have a good time? Please don't do this.

Of course, anyone else who touches the database probably won't be as enthusiastic about our creative choice of operator symboles, so we're going to stick to the standard ones.

## Building our operator class

We need to create an operator class for our field value type that provides the strategies and support functions required by the btree index method. However, before we can create our operator class or any operators at all, we need to implement functions that will compare our field_values correctly. 

Postgres has a built in procedural language, PL/pgSQL, that we can use to implement these functions and make them available to the database. We could also write our functions in C, build a shared library, and dynamically link the C code to Postgres. Using PL/pgSQL introduces fewer cross system variables however, so I am going to use it for this series. It would be an interesting exercise to see if there are significant performance differences between the two languages though!



### Generic comparison function

Before we implement comparison operators for field_value we will first write a generic comparison function "field_value_cmp". Our comparison function will take two field_values, **a** and **b**, and return -1 if **a** is less than **b**, 0 if **a** equals **b**, and 1 if **a** is greater than **b**.

We'll start by writing some helper functions to compare the native types that field_value is composed of. These functions all do pretty much the same thing, but for different input types. 

```plsql
create or replace function boolean_cmp(a boolean, b boolean) returns int2 as $$
begin
  if a < b then
    return -1;
  elsif a > b then
    return 1;
  else
    return 0;
  end if;
end;
$$LANGUAGE plpgsql;

create or replace function integer_cmp(a int8, b int8) returns int2 as $$
begin
  if a < b then
    return -1;
  elsif a > b then
    return 1;
  else
    return 0;
  end if;
end;
$$LANGUAGE plpgsql;

create or replace function float_cmp(a float8, b float8) returns int2 as $$
begin
  if a < b then
    return -1;
  elsif a > b then
    return 1;
  else
    return 0;
  end if;
end;
$$LANGUAGE plpgsql;

create or replace function string_cmp(a text, b text) returns int2 as $$
begin
  if a < b then
    return -1;
  elsif a > b then
    return 1;
  else
    return 0;
  end if;
end;
$$LANGUAGE plpgsql;

create or replace function bytes_cmp(a bytea, b bytea) returns int2 as $$
begin
  if a < b then
    return -1;
  elsif a > b then
    return 1;
  else
    return 0;
  end if;
end;
$$LANGUAGE plpgsql;

```

Next we need to implement a special comparison function for field_values with numeric types. We will first add another helper function to compare integers and floats

```plsql
create or replace function integer_float_cmp(a int8, b float8) returns int2 as $$
begin
  if a > b then
    return 1;
  else
    return -1;
  end if;
end;
$$LANGUAGE plpgsql;

```

Then we can create a function to compare two field_values that are both numeric types.

```plsql
create or replace function numeric_field_value_cmp(a field_value, b field_value) returns int2 as $$
begin
  if a.integer_value is not null then
    if b.integer_value is not null then
      return integer_cmp(a.integer_value, b.integer_value);
    else
      return integer_float_cmp(a.integer_value, b.double_value);
    end if;
  else
    if b.integer_value is not null then
      return integer_float_cmp(b.integer_value, a.double_value) * -1;
    else
      return float_cmp(a.double_value, b.double_value);
    end if;
  end if;
end;
$$LANGUAGE plpgsql;
```

Note that our helper function integer_float_cmp considers an integer that is numerically equal to a float to still be the smaller value. Telling Postgres to distinguish an integer and float that have the same numeric value in will allow us to enable deduplication of our field_values (an important database optimization) without losing type information or introducing undefined behavior. <note: make sure I understand b trees and leaf nodes and why this is important>

However, when a user makes a query, we want to consider two numerically equivalent values to be equal regardless of whether they are an integer or float. We will need to adapt the way we query numeric types internally to meet this requirement, but we will worry about that later.



Now we will implement a comparison function for field_values that hold a timestamp type.

```plsql
-- NOTE: A field_value type with timestamp_nanos >= 10^9 is considered invalid
create or replace function timestamp_field_value_cmp(a field_value, b field_value) returns int2 as $$
begin
  if a.timestamp_seconds = b.timestamp_seconds then
    return integer_cmp(a.timestamp_nanos, b.timestamp_nanos);
  else
    return integer_cmp(a.timestamp_seconds, b.timestamp_seconds);
  end if;
end;
$$LANGUAGE plpgsql;
```



And with that we have all of the comparison operators we need to compare the individual scalar types field_value can hold. Now let's make a function that will compare any two field_values with matching types

```plsql
create or replace function d(a field_value, b field_value) returns int2 as $$
begin
  if a.null_value is not null then
    return 0;
  elsif a.boolean_value is not null then
    return boolean_cmp(a.boolean_value, b.boolean_value);
  elsif a.integer_value is not null or a.double_value is not null then
    return numeric_field_value_cmp(a ,b);
  elsif a.timestamp_nanos is not null and a.timestamp_seconds is not null then
    return timestamp_field_value_cmp(a, b);
  elsif a.string_value is not null then
    return string_cmp(a.string_value, b.string_value);
  elsif a.bytes_value is not null then
    return bytes_cmp(a.bytes_value, b.bytes_value);
  else
    return string_cmp(a.reference_value, b.reference_value);
  end if;
end;
$$LANGUAGE plpgsql;
```



We also need to compare field_values that do not have matching types. To help with this we will make a function that assigns an number to the field value based on the type it represents

```plsql
create or replace function get_field_value_type(a field_value) returns int2 as $$
begin
  if a.null_value is not null then
    return 0;
  elsif a.boolean_value is not null then
    return 1;
  elsif a.integer_value is not null or a.double_value is not null then
    return 2;
  elsif a.timestamp_nanos is not null and a.timestamp_seconds is not null then
    return 3;
  elsif a.string_value is not null then
    return 4;
  elsif a.bytes_value is not null then
    return 5;
  else
    return 6;
  end if;
  return -1;
end;
$$LANGUAGE plpgsql;
```

Then, in our final comparison function, we will first compare the field_values' types and only compare their values if the types match.

```plsql
CREATE OR REPLACE FUNCTION field_value_cmp(a field_value, b field_value) RETURNS integer AS $$
declare
  a_value_type int2;
  b_value_type int2;
begin
  a_value_type := get_field_value_type(a);
  b_value_type := get_field_value_type(b);
  if a_value_type < b_value_type then
    return -1;
  elsif a_value_type > b_value_type then
    return 1;
  else
    return matching_field_value_cmp(a,b);
  end if;
end;
$$LANGUAGE plpgsql;
```

Here we check if the two field_values represent the same type. If they do, we compare their values, and if they don't we return their order based on the type order specified in <defining our requirements>



### Specific comparison functions

Using our generic comparison function as a base, we can easily implement the specific comparison functions that our operators will use

```plsql
-- Less than
CREATE OR REPLACE FUNCTION field_value_lt(a field_value, b field_value) RETURNS boolean AS $$
BEGIN
  RETURN field_value_cmp(a, b) = -1;
END;
$$LANGUAGE plpgsql;

-- Less than or equal to
CREATE OR REPLACE FUNCTION field_value_lte(a field_value, b field_value) RETURNS boolean AS $$
BEGIN
  RETURN field_value_cmp(a, b) != 1;
END;
$$LANGUAGE plpgsql;

-- Greater than
CREATE OR REPLACE FUNCTION field_value_gt(a field_value, b field_value) RETURNS boolean AS $$
BEGIN
  RETURN field_value_cmp(a, b) = 1;
END;
$$LANGUAGE plpgsql;

-- Greater than or equal to
CREATE OR REPLACE FUNCTION field_value_gte(a field_value, b field_value) RETURNS boolean AS $$
BEGIN
  RETURN field_value_cmp(a, b) != -1;
END;
$$LANGUAGE plpgsql;

-- Equal
CREATE OR REPLACE FUNCTION field_value_eq(a field_value, b field_value) RETURNS boolean AS $$
BEGIN
  RETURN field_value_cmp(a, b) = 0;
END;
$$LANGUAGE plpgsql;

-- Not equal
CREATE OR REPLACE FUNCTION field_value_neq(a field_value, b field_value) RETURNS boolean AS $$
BEGIN
  RETURN field_value_cmp(a, b) != 0;
END;
$$LANGUAGE plpgsql;

```

### Operators

Now that we have a dedicated comparison functions, we can implement operators for field_value. Lets look at the "less than" operator first.

```postgresql
CREATE OPERATOR < (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_lt,
    commutator = >,
    negator = >=,
    restrict = scalarltsel,
    join = scalarltjoinsel
);
```

This command tells the Postgres that we are defining a new operator mapped to the "<" symbol.

The parameters `function`, `leftarg`, and `rightarg` indicate that the operator will call our newly defined field_value_lt function, and that the left and right arguments of the operator will be of type field_value. The left and right arguments of the "<" operator will be passed to **field_value_lt's** **a** and **b** arguments respectively.

The parameters **commutator**, **negator**, **restrict**, and **join** tell postgres how our new operator relates to other operators (which we will define shortly) and helps it optimize the use of our operators internally. The [postgres docs](https://www.postgresql.org/docs/current/xoper-optimization.html) provide an execellent explanation of what these parameters do, so I will not repeat that explanation here. In case you don't want to read that doc page though, I will just note that `scalarltsel` and `scalarltjoinsel` are built in functions that are recommended defaults for the restrict and join parameters on a "less than" operator. 

Now let's create the rest of our operators

```postgresql

CREATE OPERATOR <= (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_lte,
    commutator = >=,
    negator = >,
    restrict = scalarlesel,
    join = scalarlejoinsel
);

CREATE OPERATOR > (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_gt,
    commutator = <,
    negator = <=,
    restrict = scalargtsel,
    join = scalargtjoinsel
);

CREATE OPERATOR >= (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_gte,
    commutator = <=,
    negator = <,
    restrict = scalargesel,
    join = scalargejoinsel
);

CREATE OPERATOR = (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_eq,
    commutator = =,
    negator = !=,
    restrict = eqsel,
    join = eqjoinsel,
    merges
);

CREATE OPERATOR != (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_neq,
    commutator = !=,
    negator = =,
    restrict = neqsel,
    join = neqjoinsel
);
```

###  The Operator Class

With all of our functions and operators defined, we can now run the command to define our operator class 

```postgresql
create operator class field_value_ops default for type field_value
using btree as
  OPERATOR 1 < ,
  OPERATOR 2 <= ,
  OPERATOR 3 = ,
  OPERATOR 4 >= ,
  OPERATOR 5 > ,
  FUNCTION 1 field_value_cmp(field_value, field_value)
;
```

The database will now use our user defined comparison operators to compare field_value types and build a btree index for field_value columns! Let's test this out.

<add deduplication>



First we'll make a throw away table with a field_value column, field_values

```postgresql
create table field_test (
    field_values      field_value
);
```

and insert some rows into it

```
insert into field_value_test values
(cast((NULL, NULL, NULL, NULL, 234, 12, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, 5, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, true, NULL, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, false, NULL, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, bytea '\xFF48AB21', NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, 0, 12, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, -999, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, -3.432, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, -3.0, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, 0, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, -0.0, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, -23.63, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, 432, 0, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, 1.0, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, '', NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, 5.0, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'mycollection/doc2') as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, bytea '\x048AB21FDA', NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, 'Hello World', NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, 1234, 12, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, 'hello world', NULL, NULL) as field_value)),
(cast((true, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'mycollection/doc1') as field_value)),
(cast((NULL, NULL, -3, NULL, NULL, NULL, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, NULL, NULL, 1234, 12, NULL, NULL, NULL) as field_value)),
(cast((NULL, NULL, 543, NULL, NULL, NULL, NULL, NULL, NULL) as field_value))
;

```

Then we'll select the field_values column in sorted order

```postgresql
select * from field_value_test order by values;
```

which results in the output
<Todo: fix bug in numeric sort>
```postgresql
           field_values            
-----------------------------
 (t,,,,,,,,)
 (,f,,,,,,,)
 (,t,,,,,,,)
 (,,-999,,,,,,)
 (,,,-23.63,,,,,)
 (,,,-3.432,,,,,)
 (,,-3,,,,,,)
 (,,,-3,,,,,)
 (,,,0,,,,,)
 (,,0,,,,,,)
 (,,,1,,,,,)
 (,,5,,,,,,)
 (,,,5,,,,,)
 (,,543,,,,,,)
 (,,,,432,0,,,)
 (,,,,0,12,,,)
 (,,,,234,12,,,)
 (,,,,1234,12,,,)
 (,,,,1234,12,,,)
 (,,,,,,"",,)
 (,,,,,,"Hellow World",,)
 (,,,,,,"hello world",,)
 (,,,,,,,"\\x048ab21fda",)
 (,,,,,,,"\\xff48ab21",)
 (,,,,,,,,mycollection/doc1)
 (,,,,,,,,mycollection/doc2)

```

We can see that all of our rows are sorted correctly! 

Let's also verify that we can add an index to the table

```postgresql
create index values_idx on field_value_test using btree (values);
set enable_seqscan=false;
```

I set enable_seqscan to false so that postgres will use our index even though it is not necessary for a table this small. And running explain on our previous query indicates that the index is being used.

```postgresql
explain select * from field_values_test
```

```
                                     QUERY PLAN                                      
-------------------------------------------------------------------------------------
 Index Only Scan using values_idx on field_test  (cost=0.14..12.53 rows=26 width=32)
```



Now we have a fully functional field_value type that can hold any of our document data types, and we are ready to implement queries on our documents.

