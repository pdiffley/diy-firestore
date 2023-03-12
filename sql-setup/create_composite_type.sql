CREATE TYPE "Unit" AS ENUM ('Exists');

CREATE TYPE field_value AS (
  min               "Unit",
  null_value        "Unit",
  boolean_value     BOOLEAN,
  integer_value     INT8,
  double_value      FLOAT8,
  timestamp_nanos   INT8,
  timestamp_seconds INT8,
  string_value      TEXT,
  bytes_value       BYTEA,
  reference_value   TEXT,
  max               "Unit"
);

-- BASIC COMPARISON FUNCTIONS

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


-- COMPOSITE COMPARISON FUNCTIONS


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


-- MATCHING FIELD COMPARISON
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

-- FIELD VALUE TYPE IDENTIFICATION
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

-- FINAL COMPARISON FUNCTION
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



-- OPERATORS CAN BE OVERLOADED WITH DIFFERENT TYPES
CREATE OPERATOR < (
    leftarg = field_value,
    rightarg = field_value,
    function = field_value_lt,
    commutator = >,
    negator = >=,
    restrict = scalarltsel,
    join = scalarltjoinsel
);

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


CREATE OPERATOR CLASS field_value_ops DEFAULT for TYPE field_value
USING btree AS
  OPERATOR 1 < ,
  OPERATOR 2 <= ,
  OPERATOR 3 = ,
  OPERATOR 4 >= ,
  OPERATOR 5 > ,
  FUNCTION 1 field_value_cmp(field_value, field_value);
