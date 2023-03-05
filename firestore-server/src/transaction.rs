use postgres::Transaction;

use crate::composite_query::CompositeFieldGroup;
use crate::protos::document_protos::Document;
use crate::security_rules::UserId;
use crate::write::{delete_document, write_document};

pub struct TransactionOperationValue {
  operation: TransactionOperation,
  document: Document,
  relevant_composite_groups: Vec<CompositeFieldGroup>,
}

pub enum TransactionOperation {
  Write,
  Delete,
}

pub fn commit_transaction(sql_transaction: &mut Transaction, user_id: &UserId, read_documents: &[Document], write_operations: &[TransactionOperationValue]) -> bool {
  for document in read_documents {
    if document_has_changed(sql_transaction, document) {
      return false;
    }
  }

  for operation in write_operations {
    match operation.operation {
      TransactionOperation::Write => write_document(sql_transaction, user_id, operation.document.clone(), &operation.relevant_composite_groups),
      TransactionOperation::Delete => {
        let collection_parent_path = operation.document.id.clone().unwrap().collection_parent_path;
        let collection_id = operation.document.id.clone().unwrap().collection_id;
        let document_id = operation.document.id.clone().unwrap().document_id;
        delete_document(sql_transaction, user_id, &collection_parent_path, &collection_id, &document_id, &operation.relevant_composite_groups)
      }
    }
  }

  true
}

fn document_has_changed(transaction: &mut Transaction, document: &Document) -> bool {
  let collection_parent_path = document.id.clone().unwrap().collection_parent_path;
  let collection_id = document.id.clone().unwrap().collection_id;
  let document_id = document.id.clone().unwrap().document_id;
  let update_id = document.update_id.clone();

  return if let Some(update_id) = update_id {
    transaction.query(
      "SELECT 1 FROM documents WHERE collection_parent_path=$1 and collection_id=$2 and document_id=$3 and update_id=$4",
      &[&collection_parent_path, &collection_id, &document_id, &update_id],
    ).unwrap().len() == 0
  } else {
    transaction.query(
      "SELECT 1 FROM documents WHERE collection_parent_path=$1 and collection_id=$2 and document_id=$3",
      &[&collection_parent_path, &collection_id, &document_id],
    ).unwrap().len() == 0
  };
}
