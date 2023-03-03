
CREATE TABLE test_table (
  myvar  int8
);
create index testindex1 on test_table(myvar);

CREATE TABLE field_test_table(
  myvar   field_value
);
create index field_testindex1 on field_test_table(myvar);

CREATE TABLE documents (
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  document_data               bytea,
  update_id                   text
);

create index collection_id_collection_parent_path_index 
on documents(collection_id, collection_parent_path, document_id, update_id);

CREATE TABLE basic_subscriptions (
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  subscription_id             text
);

create index basic_subscriptions_idx 
on basic_subscriptions(collection_parent_path, collection_id, document_id);

CREATE TABLE simple_query_lookup (
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  field_name                  text,
  field_value                 field_value,
  primary key (collection_parent_path, collection_id, document_id, field_name)
);

create index simple_query_idx on simple_query_lookup(collection_id, field_name, field_value, collection_parent_path);
create index simple_query_deletion_idx on simple_query_lookup(collection_parent_path, collection_id, document_id);

CREATE TABLE client_subscriptions (
  subscription_id     text,
  client_id           text
);

create index client_subscriptions_subscription_id_idx on client_subscriptions(subscription_id);
create index client_subscriptions_client_id_idx on client_subscriptions(client_id);

CREATE TABLE simple_query_subscriptions (
  collection_parent_path      text,
  collection_id               text,
  field_name                  text,
  field_operator              text,
  field_value                 field_value,
  subscription_id             text
);

create index simple_query_subscription_idx on 
simple_query_subscriptions(collection_parent_path, collection_id, field_name, field_operator, field_value);


CREATE TABLE update_queues (
  subscription_id             text,
  collection_parent_path      text,
  collection_id               text,
  document_id                 text,
  document_data               bytea,
  update_id                   text
);

create index update_queues_subscription_id_idx on update_queues(subscription_id);
create index update_queues_update_id_idx on update_queues(update_id);
