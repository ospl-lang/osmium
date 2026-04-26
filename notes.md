Implementing functions:

Say we want to make a basic counter that can be added and subtracted from...

Now, we can do the really fucking easy thing and just...
```ospl
def num: Int = 0;
num += 1;
num -= 1;
```

But let's say, for whatever reason, we want to encapsulate this like
we're React.JS devs writing a button.

We can do
```ospl
def Counter = fn(initial: Int) {
    def hold x: Int = initial;  # 'hold' makes it private

    # 'let' makes it public
    def let inc = fn(): Int {
        x += 1;
        return x
    }

    def let dec = fn(): Int {
        x -= 1;
        return x
    }

    # unsure of the syntax for this.. but just know, we're gonna
    # need syntax to return a `Scope`
}
```

But this doesn't help us, we have no access to `inc` or `dec`.
...or do we?

The VM has a datatype called `Frame`, this stores a stack frame
(which basically just contains local variables). We can wrap the
`Frame` type in a `Value` (or `RuntimeValue`??)

Each function has its own frame when it gets called...
`def c = Counter(0);`

Now, since we returned a `Scope`, we can now do
`c.inc()` and `c.dec()`, yippie!!!