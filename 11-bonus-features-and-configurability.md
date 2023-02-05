
queries on simple fields disabled by default
  - dramatically reduces write cost
  - only requires minimal effort to turn on simple queries for a field

Specify composite query that you want to make in advance rather than just the fields you want to make a composite query over
  - more efficient, and removes a lot of restrictions on what type of composite queries you can make

  eg. instead of telling the database in advance that you want to make composite queries over the fields "age", "name", and "school" you'd tell the database that you want to make a query over:
    "age" with the ">" operator
    "name" with the "=" operator and
    "school" with the "=" operator

Configure the ttl for individual types of subscription queues 
  eg. if you have a messaging app with a messages collection, you could indicate server side that you want subscriptions to your messages collection to not expire for (example 10 days) after the client disconnects. Then you can have you app subscribe directly to a collection of messages (where every message is a document), and not have to worry about the app having to reload every single message ever sent just because a user was offline for 30 minutes.