# intercourse-rs
What Rust-things I desperately call for when using golang ...

This is the technical backbone of a presentation about Rust after switching back
to golang in a professional setup. Sometimes, I'm stupid and I make
**mistakes**. I can't speak for others, but I prefer to make my mistakes in
Rust. Here is why ...

## Overview & Introduction

This repository is a collection of hands-on sample code that demonstrate aspects
of Rust that I definitively miss when coding in golang. The list of such aspects
include:

* Language Concepts
  * Sum types (Algebraic data types)
  * RAII
  * Iterators
  * Scoping
  * Ownership types
  * Send + Sync
* Tooling
* Documentation

## Sum types

_Together with product types (`struct`s and tuples), sum types allow for
**unambiguous** modelling of application state._

The key in the above sentence is «unambiguous». Consider the following
enumeration that represents the state of a Job, that is either waiting for
execution, being processed or has finished with a result. Each state of a value
of type `JobState` has a well-defined meaning:

```rust
enum JobState<T> {
    Queued,
    Processing,
    Finished { result: T }
}
```

Here is a trivial example of how a value of type `JobState` could be
deconstructed:

```rust
fn print_job_state<T: Debug>(js: JobState<T>) {
    match js {
        Queued              => println!("job is queued"),
        Processing          => println!("job is being processed"),
        Finished { result } => println!("yay! we have a result: {:?}", result)
    }
}
```

Now, compare this to how this would be represented in golang:

```golang
const (
    Queued     = iota
    Processing
    Finished
)

struct JobState[T] {
    Result     *T   // possibly nil
    State      int
}
```

The `.Result` field could be set, even though the state indicates `Queued` or
`Processing`. Conversely, the result could be set to `nil`, even though `.State`
is set to `Finished` (`nil`-pointer propagation). As a result, one has to resort
to _informal conventions_ to ensure consistency across API-boundaries and _hope_
that the implementor of an API adheres to those conventions.

It's worth noting that algebraic data types (ADTs) first appeared in
strongly-typed functional languages ([Haskell](https://www.haskell.org/),
[Ocaml](https://ocaml.org/), [Scala](https://www.scala-lang.org/), ...) and then
made their way into [Swift](https://www.swift.org/),
[Kotlin](https://kotlinlang.org/), [Elm](https://elm-lang.org/),
[Typescript](https://www.typescriptlang.org/), ... and
[Rust](https://www.rust-lang.org/).

### `Option<T>`

A canonical example of a sum type in Rust is
[`Option<T>`](https://doc.rust-lang.org/std/option/enum.Option.html), which is
used wherever absence of a value is possible:

```rust
pub enum Option<T> {
    None,
    Some(T),
}
```

Needless to say, Option-values are all over the place. A common situation is
when one wants to retrieve a value from a collection. Let's say, e.g., we have a
hash map `prefs` that we use to retrieve people's preferences based on their
name:

```rust
// Assume prefs is of type: HashMap<String, Prefs>
let alice_prefs = prefs.get("Alice");

if let Some(prefs) = alice_prefs {
    println!("Alice prefs are: {alice_prefs:?}")
} else {
    println!("Ohoh! Alice has no prefs!")
}
// ==== an alternative to the above way of pattern matching is =========
match alice_prefs {
    Some(alice_prefs) => println!("Alice's prefs are: {alice_prefs:?}"),
    None => println!("Ohoh! Alice has no prefs!")
}
```

In the above example, `HashMap::get` returns a value of type `Option<&Prefs>`.
So, it returns a reference to the object _owned_ by the hash map (if we wanted
to own the value, we would need to move it out of the hash map using `.remove`).

Often times, we _assert statically_ that a value is actually contained in a
collection. In this case, we can just call
[`.unwrap()`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap)
on the `Option` and use the contained value:

```rust
let alice_prefs = prefs.get("Alice").unwrap();
println!("Alice prefs are: {alice_prefs:?}")
```

The call to `.unwrap()` panics if the underlying value is `None`. In general,
there is no guarantee of course that the sought after value is actually
contained. But the key here is that a user of the API has no choice but to deal
with potential absence and the panic generally happens at the earliest (!)
possible point where a mistaken assumption is made—not somewhere downstream.

Compare this to the situation in golang:

```golang
prefs := make(map[string]Prefs)
// ...
alice_prefs := prefs["Alice"]
```

In the above example, if `"Alice"` is missing, `alice_prefs` is just initialized
to the zero-value of the struct and the value propagates happily throughout the
code—most of the time, not what we want.

> Note: Depending on the situation, it is preferrable to use `.expect("some
message")` instead of just `.unwrap()`. The given message will be included in
the panic message and provides further guidance in case of a panic. It also acts
as documentation on why a value is expected to be there.

#### Intermezzo: `Option::<&T>::None == null`

This is a quick intermezzo regarding the all-farmous «zero-cost abstractions» in
Rust. In the above example, the call to
[`get`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get)
returns a `Option<&V>`, where `V` is the value type of the hash map. In this
case, and whenever `Option` just wraps a reference, there is no overhead
associated with using an option type at runtime. As a null-pointer is an invalid
reference in any case, the `None`-variant is simply represented as a
`null`-Pointer.

### CLI-Argument parsing: a more elaborate use case

In my not-so-humble view, the easy stuff is also easy in Rust—but simpler(*). While
the hard stuff is becoming possible. And this section, I would like to discuss a
quick example of this, demonstrating another concrete use case of ADTs in rust:
CLI-argument parsing.

(*) Note: simple and easy are not the same thing.

ADTs are a good fit to represent abstract syntax trees (ASTs) in general. And in
Rust, one can reprsent the set of possible input arguments to a program as a
type. In other words: there is no need to figure out the structure of the parser
by reading program code—it is informed by the type declaration.

These days, the de-facto standard library in Rust to parse command line
arguments [`clap`](https://docs.rs/clap/latest/clap/). There are alternatives,
but unless you have specific needs, just use clap.

It is possible to assemble the parser in a programmatic way using a builder
pattern. And in fact, this is what is happening any way. However, the clap
authors went through the pain of providing macro-annotations that create the
parser based upon a structure:

```rust
/// A simple program that takes an optional -v/--verbose argument and a
/// filename.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Verbose output.
    #[arg(short, long)]
    verbose: bool,
    
    // A simple positional argument ...
    /// The file to be processed.
    path: PathBuf,
}

fn main() {
    let cli = Args::parse();
}
```

In the above example, the function `parse()` acts as a _constructor_ of a value
of type `Args`. It either succeeds in doing so based on the command-line
arguments passed to the program or fails with an error message. In the latter
case, it would look something like this:

```text
error: the following required arguments were not provided:
  <PATH>

Usage: executable <PATH>

For more information, try '--help'.
```

