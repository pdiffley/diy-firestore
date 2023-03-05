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

Fortunately, Postgres has a built in way for us to implement custom comparison operators. Postgres also has some specific and fine grained language for how operators, their underlying functions, and indexes relate to each other and that terminology is not always intuitive. Let's go over that now so we are all on the same page.

### Terminology

**Operator**

An operator is the symbol representing some operation (eg. ">", "<" "="). It is the symbol you put in a "where" clause to limit the scope of a query. 

Postgres allows us to define a new operator and associate it with a function that will be called when the operator is used. Operators can be overloaded for different types, so we can use standard operator symbols for our custom type. We can use the "<" symbol to perform a "less than" operation for example.

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

Before we implement out operators, I'd like point out again that because of the way postgres maps operators to indexing "strategies", we did not have to use the operator symbols shown in the command above. Even though the strategy number **1** expects a "less than" operation we did not need to assign that strategy an operator with the "<" symbol. As long as the operator we assign the to strategy 1 performs a "less than" comparison for two field values, the index is happy.

So if you were feeling particularly excited one day and wanted to pass that enthusiam along to your database, you could assign the "equal" strategy the operator "=D". Fun right? Who ever said database engineers don't know how to have a good time? Please don't do this.

## Building our operator class

We need to create an operator class for our field value type that provides the strategies and support functions required by the btree index method. However, before we can create our operator class or any operators at all, we need to implement functions that will compare our field_values correctly. 

Postgres has a built in procedural language, PL/pgSQL, that we can use to implement these functions and make them available to the database. We could also write our functions in C, build a shared library, and dynamically link the C code to Postgres. Using PL/pgSQL introduces fewer cross system dependencies however, so I am going to use it for this series. It would be an interesting exercise to see if there are significant performance differences between the two languages though!

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
$$language plpgsql;

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
$$language plpgsql;

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
$$language plpgsql;

create or replace function integer_float_cmp(a int8, b float8) returns int2 as $$
begin
  if a > b then
    return 1;
  elsif a > b then
    return 1;
  else
    return 0;
  end if;
end;
$$language plpgsql;

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
$$language plpgsql;

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
$$language plpgsql;

```

Next we need to implement a special comparison function for field_values with numeric types. 

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
$$language plpgsql;
```

Now we will implement a comparison function for field_values that hold a timestamp type.

```plsql
-- NOTE: A field_value type with timestamp_nanos >= 10^9 is considered invalid and an error
create or replace function timestamp_field_value_cmp(a field_value, b field_value) returns int2 as $$
begin
  if a.timestamp_seconds = b.timestamp_seconds then
    return integer_cmp(a.timestamp_nanos, b.timestamp_nanos);
  else
    return integer_cmp(a.timestamp_seconds, b.timestamp_seconds);
  end if;
end;
$$language plpgsql;
```

And with that we have all of the comparison operators we need to compare the individual scalar types field_value can hold. Now we can make a function that will compare any two field_values that have matching types

```plsql
create or replace function matching_field_value_cmp(a field_value, b field_value) returns int2 as $$
begin
  if a.min is not null then
    return 0;
  elsif a.null_value is not null then
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
  elsif a.reference_value is not null then
    return string_cmp(a.reference_value, b.reference_value);
  else
    return 0;
  end if;
end;
$$language plpgsql;
```

We also need to compare field_values that do not have matching types. To help with this we will make a function that assigns an number to the field value based on the type it contains

```plsql
create or replace function get_field_value_type(a field_value) returns int2 as $$
begin
  if a.min is not null then
    return 0;
  elsif a.null_value is not null then
    return 1;
  elsif a.boolean_value is not null then
    return 2;
  elsif a.integer_value is not null or a.double_value is not null then
    return 3;
  elsif a.timestamp_nanos is not null and a.timestamp_seconds is not null then
    return 4;
  elsif a.string_value is not null then
    return 5;
  elsif a.bytes_value is not null then
    return 6;
  elsif a.reference_value is not null then
    return 7;
  else
    return 8;
  end if;
  return -1;
end;
$$language plpgsql;

```

Then, in our final comparison function, we will first compare the field_values' types and only compare their values if the types match.

```plsql
create or replace function field_value_cmp(a field_value, b field_value) returns integer AS $$
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
$$language plpgsql;
```

### Specific comparison functions

Using our generic comparison function as a base, we can easily implement the specific comparison functions that our operators will use

```plsql
-- Less than
create or replace function field_value_lt(a field_value, b field_value) returns boolean AS $$
begin
  return field_value_cmp(a, b) = -1;
end;
$$language plpgsql;

-- Less than or equal to
create or replace function field_value_lte(a field_value, b field_value) returns boolean AS $$
begin
  return field_value_cmp(a, b) != 1;
end;
$$language plpgsql;

-- Greater than
create or replace function field_value_gt(a field_value, b field_value) returns boolean AS $$
begin
  return field_value_cmp(a, b) = 1;
end;
$$language plpgsql;

-- Greater than or equal to
create or replace function field_value_gte(a field_value, b field_value) returns boolean AS $$
begin
  return field_value_cmp(a, b) != -1;
end;
$$language plpgsql;

-- Equal
create or replace function field_value_eq(a field_value, b field_value) returns boolean AS $$
begin
  return field_value_cmp(a, b) = 0;
end;
$$language plpgsql;

-- Not equal
create or replace function field_value_neq(a field_value, b field_value) returns boolean AS $$
begin
  return field_value_cmp(a, b) != 0;
end;
$$language plpgsql;
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

The parameters `function`, `leftarg`, and `rightarg` indicate that the operator will call our newly defined field_value_lt function, and that the left and right arguments of the operator will be of type field_value. The left and right arguments of the "<" operator will be passed to the `a` and `b` arguments of `field_value_lt`.

The parameters `commutator`, `negator`, `restrict`, and `join` tell postgres how our new operator relates to other operators (which we will define shortly) and helps it optimize the use of our operators internally. The [postgres docs](https://www.postgresql.org/docs/current/xoper-optimization.html) provide an execellent explanation of what these parameters do, so I will not repeat that explanation here. In case you don't want to read that doc page though, I will just note that `scalarltsel` and `scalarltjoinsel` are built in functions that are recommended defaults for the restrict and join parameters on a "less than" operator. 

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
CREATE OPERATOR CLASS field_value_ops DEFAULT for TYPE field_value
USING btree AS
  OPERATOR 1 < ,
  OPERATOR 2 <= ,
  OPERATOR 3 = ,
  OPERATOR 4 >= ,
  OPERATOR 5 > ,
  FUNCTION 1 field_value_cmp(field_value, field_value);
```

The database will now use our user defined comparison operators to compare field_value types and build a btree index for field_value columns! Let's test this out.

```postgresql
SELECT * FROM ( VALUES
(cast((NULL, NULL, NULL, NULL, NULL, 234, 12, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, 5, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, true, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, false, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, bytea '\xFF48AB21', NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, 0, 12, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, -999, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, -3.432, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, -3.0, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, 0, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, -0.0, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, -23.63, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, 432, 0, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, 1.0, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, '', NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, 5.0, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'mycollection/doc2', NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, bytea '\x048AB21FDA', NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Hello World', NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, 1234, 12, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'hello world', NULL, NULL, NULL) AS field_value)),
(cast((NULL, 'Exists', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast(('Exists', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'Exists') AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'mycollection/doc1', NULL) AS field_value)),
(cast((NULL, NULL, NULL, -3, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, NULL, NULL, 1234, 12, NULL, NULL, NULL, NULL) AS field_value)),
(cast((NULL, NULL, NULL, 543, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value))) 
  AS test_table(field_value)
ORDER BY field_value ASC;
```

This query results in the output

```postgresql
          field_value          
-------------------------------
 (Exists,,,,,,,,,,)
 (,Exists,,,,,,,,,)
 (,,f,,,,,,,,)
 (,,t,,,,,,,,)
 (,,,,-23.63,,,,,,)
 (,,,-999,,,,,,,)
 (,,,,-3.432,,,,,,)
 (,,,,-3,,,,,,)
 (,,,-3,,,,,,,)
 (,,,0,,,,,,,)
 (,,,,0,,,,,,)
 (,,,,1,,,,,,)
 (,,,,5,,,,,,)
 (,,,5,,,,,,,)
 (,,,543,,,,,,,)
 (,,,,,432,0,,,,)
 (,,,,,0,12,,,,)
 (,,,,,234,12,,,,)
 (,,,,,1234,12,,,,)
 (,,,,,1234,12,,,,)
 (,,,,,,,"",,,)
 (,,,,,,,"Hello World",,,)
 (,,,,,,,"hello world",,,)
 (,,,,,,,,"\\x048ab21fda",,)
 (,,,,,,,,"\\xff48ab21",,)
 (,,,,,,,,,mycollection/doc1,)
 (,,,,,,,,,mycollection/doc2,)
 (,,,,,,,,,,Exists)
```

We can see that all of our rows are sorted correctly! 

### A note on performance

I am mostly ignoring performance issues in this series, but there is an issue with our operator class as written above that I think is significant enough to draw special attention to. Postgres typically has the ability to deduplicate indentical column values. So if you have a column of strings representing three possible categories and an index on that column, the full category string does not need to be repeated as a leaf page tuple for every row. 

The catch is that for deduplication to be enabled, the comparison function used for building an index needs to guarantee "image equality", meaning that two values which are considered equal are not just equivalent but are identical and can be interchanged without any loss of information. Because we consider a field_value with equivalent integer and floating point values to be equal (eg. we consider 1 and 1.0 to be equal), we cannot enable deduplication without losing information. 

It is possible to alter our comparison functions so that we can enable deduplication, but when then need to take that into account when writing the code to support queries. To avoid that complexity, I am just leaving deduplication off for now. I'll revisit this and go over the modifcations we would need to make to support deduplication later in the <bonus section>. If you are interested in the detail of how Postgres handles deduplication, [the docs](https://www.postgresql.org/docs/current/btree-implementation.html#BTREE-DEDUPLICATION) explain it quite well.

### Next up

We now have a fully functional field_value type that can hold any of our document data types, and we are ready to implement queries on our documents.

