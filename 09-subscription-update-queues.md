we will create a "queue" for each subscription a client has (all queues likely in one table, but tbd), when a subscription is affected by a write, the update will be written to the queue

storing our update queues in the same database as all the rest of our data means we can update our subscription queues
in the same transaction that a write is made in ensuring that we never drop any updates.

we will also specify a ttl for a subscription, so that if a client has been disconnected for a certain threshold period of time, we no longer spend the resources to maintain updates to the subscription. 

After the queue is updated we will also send a (pubsub, sqs, rabbitmq) message to the client connection server telling it to send the update to the client. This message is outside the transaction, but it is ok if it gets dropped occasionally since that server will have redundant checks on the update queue itself. 