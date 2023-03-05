give documents an update_id (in document table)



only need one extra endpoint for optimistic transaction:

Commit Transaction
takes list of documents that should have no change (based on update_id) and the list of documents to write
fails if any of the update ids have changed

Client is responsible for tracking previously read documents. 



Show example of how this works client side.

Point out requirement that reads happen before writes. 
