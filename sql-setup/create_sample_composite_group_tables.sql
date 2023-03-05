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

CREATE TABLE composite_included_table_d8b8c614b73546daa1d85531dc412ef6(
  subscription_id   TEXT,
  min_age           field_value,
  max_age           field_value,
  city              field_value,
  user_name         field_value,
  zipcode           field_value
);

CREATE INDEX composite_included_table_idx_d8b8c614b73546daa1d85531dc412ef6
ON composite_included_table_d8b8c614b73546daa1d85531dc412ef6(min_age, max_age, user_name, city, zipcode);

CREATE TABLE composite_excluded_table_d8b8c614b73546daa1d85531dc412ef6 (
  subscription_id   TEXT,
  excluded_age       field_value
);

CREATE INDEX composite_excluded_table_idx_d8b8c614b73546daa1d85531dc412ef6 
ON composite_excluded_table_d8b8c614b73546daa1d85531dc412ef6(excluded_age); 