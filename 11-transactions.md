Set up with long polling or streaming request for trivial horizontal scalability

basic flow is

  client connects with a long poll request
  subscribes to server side message queue
  checks if it's update queue for any missed messages
  if there are messages
    tell client to get the updates from the queue
  else 
    wait for an update signal then tell client to get updates from the queue

Messages won't be removed from the update queue until the client confirms the update has been processed. 
The client side update will be designed to be idempotent, so duplicate update won't cause a problem.
