CREATE TABLE composite_lookup_table_d8b8c614b73546daa1d85531dc412ef6 (
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  age                         field_value,
  city                        field_value,
  name                        field_value,
  zipcode                     field_value
);

create index composite_lookup_table_idx_d8b8c614b73546daa1d85531dc412ef6 
on composite_lookup_table_d8b8c614b73546daa1d85531dc412ef6(age, city, name, zipcode);

create table composite_included_table_d8b8c614b73546daa1d85531dc412ef6(
  subscription_id   text,
  min_age           field_value,
  max_age           field_value,
  city              field_value,
  name              field_value,
  zipcode           field_value
);

create index composite_included_table_idx_d8b8c614b73546daa1d85531dc412ef6
on composite_included_table_d8b8c614b73546daa1d85531dc412ef6(min_age, max_age, name, city, zipcode);

create table composite_excluded_table_d8b8c614b73546daa1d85531dc412ef6 (
  subscription_id   text,
  excluded_age       field_value
);

create index composite_excluded_table_idx_d8b8c614b73546daa1d85531dc412ef6 on 
composite_excluded_table_d8b8c614b73546daa1d85531dc412ef6(excluded_age); 