
CREATE TABLE documents (
  collection_parent_path      TEXT,
  collection_id               TEXT,
  document_id                 TEXT,
  document_data               BYTEA,
  update_id                   TEXT,
  PRIMARY KEY (collection_parent_path, collection_id, document_id)
);

CREATE INDEX collection_id_collection_parent_path_idx
ON documents(collection_id, collection_parent_path, document_id, update_id);

CREATE TABLE basic_subscriptions (
  collection_parent_path      TEXT,
  collection_id               TEXT,
  document_id                 TEXT,
  subscription_id             TEXT
  PRIMARY KEY (subscription_id)
);

CREATE INDEX basic_subscriptions_idx 
ON basic_subscriptions(collection_parent_path, collection_id, document_id);

CREATE TABLE simple_query_lookup (
  collection_parent_path      TEXT,
  collection_id               TEXT,
  document_id                 TEXT,
  field_name                  TEXT,
  field_value                 field_value,
  PRIMARY KEY (collection_parent_path, collection_id, document_id, field_name)
);

CREATE INDEX simple_query_idx ON simple_query_lookup(collection_id, field_name, field_value, collection_parent_path);
CREATE INDEX simple_query_deletion_idx ON simple_query_lookup(collection_parent_path, collection_id, document_id);

CREATE TABLE simple_query_subscriptions (
  collection_parent_path      TEXT,
  collection_id               TEXT,
  field_name                  TEXT,
  field_operator              TEXT,
  field_value                 field_value,
  subscription_id             TEXT,
  PRIMARY KEY (subscription_id)
);

CREATE INDEX simple_query_collection_subscription_idx ON 
simple_query_subscriptions(collection_id, field_name, field_operator, field_value, collection_parent_path);

CREATE TABLE client_subscriptions (
  subscription_id     TEXT,
  client_id           TEXT
);

CREATE INDEX client_subscriptions_subscription_id_idx ON client_subscriptions(subscription_id);
CREATE INDEX client_subscriptions_client_id_idx ON client_subscriptions(client_id);

CREATE TABLE update_queues (
  subscription_id             TEXT,
  collection_parent_path      TEXT,
  collection_id               TEXT,
  document_id                 TEXT,
  document_data               BYTEA,
  update_id                   TEXT
);

CREATE INDEX update_queues_subscription_id_idx ON update_queues(subscription_id);
CREATE INDEX update_queues_update_id_idx ON update_queues(update_id);
