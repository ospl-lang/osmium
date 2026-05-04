## Refcounts
Refcounts should ONLY increment when:
- a function is called with the object as a parameter
- a binding is made to a function return  <!-- maybe? -->

Refcounts should ONLY decrement when:
- a function returns
