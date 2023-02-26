create table field_value_test (
    values      field_value
);

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

select * from field_value_test order by values;

select * from field_value_test 
where values > cast((NULL, NULL, 1, NULL, NULL, NULL, NULL, NULL, NULL) as field_value) 
order by values;


// Use index on actual table later rather than sort test case
create index values_idx on field_test using btree (values);

//to test indexes on small database run in psql
set enable_seqscan=false;


Todo: Define actualy compostie type minus map and array
Write comparison function for it