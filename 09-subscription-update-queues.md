Now we can implement our actual client side updates.

With a naive implemenation, we would just notify the client that a subscription was updated, and have it re-query the data. 

That leads to some n^2 behavior though (rereading an entire collection everytime a document is added for example)

Instead, when a subscription is updated, we want to only update the client with documents that have changed. 

we will create a "queue" for each subscription a client has (all queues likely in one table, but tbd), when a subscription is affected by a write, the update will be written to the queue

storing our update queues in the same database as all the rest of our data means we can update our subscription queues
in the same transaction that a write is made in ensuring that we never drop any updates (we will confirm that the client has received the update before removing the item from the queue). At any point in time, the queue represents the latest state of the query, so it is ok if the client gets a redundant update (idempotent?)<rephrase to be less confusing>.

we will also specify a ttl for a subscription, so that if a client has been disconnected for a certain threshold period of time, we no longer spend the resources to maintain updates to the subscription. 

After the queue is updated we will also send a (pubsub, sqs, rabbitmq) message to the client connection server telling it to send the update to the client. This message is outside the transaction, but it is ok if it gets dropped occasionally since that server will have redundant checks on the update queue itself. 



| client_id | subscription_id | collection_parent_path | collection_id | document_id | document_data |
| --------- | --------------- | ---------------------- | ------------- | ----------- | ------------- |
| Text      | Text            | Text                   | Text          | Text        | Bytes         |

on subscription update insert row

(document data is null if the document is deleted)
