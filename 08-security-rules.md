most basic verison of security rules is a function implemented with the rest of the code
called whenever a document is read, written or listed
args: doc id, operation type, document
returns bool for whether operation is allowed

most flexible and performant

flexibility could be a draw back, because you don't have a structure to base your rules off of

Other approach would be to run a sidecar server with the security rules
  - makes it easy to write your security rules in your frontend language of choice

Writing your security rules in an existing programming language has a couple big benefits
  - useful compilation errors. Writing rules in a config file can be error prone and error messages are almost always inferior to those of an actual programming language
  - You can unit test your security rule function with existing test suites

Draw back is you have to deploy your security rules like code rather than as a config upload, but that isn't a problem for 
us since we are already deploying the whole database ourselves for just our use.

Could also make a config file set up like firestore uses, but then you lose the ability to get useful compilation errors

If you like the structure of Firestore's security rules, it would not be hard to make a wrapper function to process a list of regex rules