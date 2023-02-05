
pub enum Operation {
  Get, List, Create, Update, Delete
}

pub enum UserId {
  Admin,
  User(Option<String>),
}

pub fn operation_is_allowed(user_id: &Option<String>, operation: &Operation, collection_parent_path: &Option<String>,
                            collection_id: &str, document_id: &Option<String>) -> bool {
  // if the user is authenticated allow access
  if let Some(user_id) = user_id {
    return true;
  }
  false
}

