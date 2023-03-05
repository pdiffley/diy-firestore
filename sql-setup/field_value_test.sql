
CAST((NULL, NULL, NULL, 5, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)

SELECT * FROM ( VALUES
	(CAST((NULL, NULL, NULL, 2, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
	(CAST((NULL, NULL, NULL, 1, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
	(CAST((NULL, NULL, NULL, NULL, 1.0, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value))) 
  AS test_table(field_value)
ORDER BY field_value ASC;

SELECT * FROM ( VALUES
	(CAST(('Exists', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
	(CAST((NULL, NULL, NULL, 1, NULL, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value)),
	(CAST((NULL, NULL, NULL, NULL, 1.0, NULL, NULL, NULL, NULL, NULL, NULL) AS field_value))) 
  AS test_table(field_value)
ORDER BY field_value ASC;


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


