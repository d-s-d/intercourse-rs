# intercourse-rs

This repo is the technical backbone of a presentation about core Rust
concepts—concepts I would call for when working in another language. The repo
contains a number of individual crates that are organized in a cargo workspace.

This README.md introduces the aforementioned concepts on an abstract level and
references the crates for exemplification. In some places—and I beg for
forgiveness—I will compare examples to corresponding equivalent golang
implementations.

# Installing Rust

Make sure you have rust and cargo
[installed](https://www.rust-lang.org/tools/install) on your system.

Once you have cargo installed, you can run general commands like `cargo clippy`,
`cargo test`, or `cargo build` in the root directory and run the linter, the
test or the build is executed for all contained crates.

_For example_:
```console
intercourse-rs/ $ cargo build
...
     Finished dev [unoptimized + debuginfo] target(s) in 0.60s
```

If you want to run a binary of a specific crate, you can either specify the
crate name when calling `cargo run --bin <crate_name>` from the root directory,
or switch into the corresponding crate directory and just run `cargo run` there.
The crate name is specified in the `Cargo.toml` of the respective crate.

```console
# the following two commands yield equivalent results
intercourse-rs/ $ cargo run --bin <crate_name>
intercourse-rs/ $ pushd <dir_of_crate>; cargo run --bin <crate_name>; popd
```

# Introduction & Overview

## Why Rust ...

*sigh*

> You don't know a programming language — unless you *hate* it ... 😉
>
> — *yours truly*

You can trust me on one thing: Rust can be a nasty. And especially in areas
where I myself carried the risk for operational problems myself (e.g., you need
to restart a job on CI because out-of-memory) and _not_ the customer, every now
and then I found yourself thinking: _Jesus, I don't wanna bother with concurrent
memory management, just give me a high-level, functional, GC-collected language
already!_

But just as the grass is always greener on the other side, you only truly learn
what something is worth after you lose it. I don't have a favorite programming
language and, at this point, I think it's unwise thing to have that anyway.
However, under specific constraints, I'm willing to make strong bets. And one of
these strong bets is the following: If my «bread and butter» boils down to
delivering and maintaining mission critical components of (distributed) systems,
Rust is my first (and only) choice. _Life is short in systems engineering as
well, and mistakes happen. I prefer to make my mistakes in Rust. Here is why
..._

## What: the theory ...

Using expressive, strong type systems one can typically reveal defects at an
_early_ stage of the implementation; defects which would otherwise «slip
through» and potentially cause havoc at runtime. However, the fact that we can
check consistency mechanically is only part of the story: through the use of
strong types, we provide more accurate information about what we are trying to
achieve, not only to the compiler—but also to ourselves _and others_. As a
result, in my experience, many potential issues become obvious already at the
_modeling stage_—and thus _before_ we run the type checker even the first time.

For example: Rust does not allow you to just share pointers across threads and
the type system makes that obvious. If you want to have a smart pointer, you
have to decide whether it is just reference-counted (`Rc`) or atomically
reference-counted (`Arc`). The type system asks you, so to speak, _are you sure
you don't want to share across threads?_ (In most situations, you just use an
`Arc` and be done with it.)

Furthermore, while type-sophistication is _also_ a way to make reviewers'
lives—let's say—interesting, strong types tend to reduce the risk of
misunderstandings between engineers. And I'm sorry for being provocative here,
but especially the latter point I cannot stress enough when claims come up about
how certain «simple» programming languages are supposedly making the lives of
reviewers comparatively easier.

Also—and again, my personal view—what it feels like to write code in a
programming language is really _secondary_ to another question: _How quickly can
I convince myself that a specific property holds for a given code?_ To give an
example: while the experience of _writing_ Python code is quite pleasant, the
experience of operationalizing it and taking responsibility for the reliability
of a shipped/deployed system component written in Python is anything but
pleasant.

Conversely, in Rust, at the moment you write some code, you might be annoyed
that you need to decide, e.g., «`Arc` or `Rc`?» But in the long run, _others_
(including your _future self_) will highly appreciate the fact that this
decision is clearly documented and the consequences of it are statically and
mechanically enforced. And this is even more true if the decision turns out to
be the wrong one down the road. Because again, the type system prevents you from
violating prior assumptions.

## Why it matters: Engineering must scale ...

> Conventions don't scale.
>
> — Bjarne Stroustrup, [«Delivering Safe C++»](https://www.youtube.com/watch?v=I8UvQKvOSSw), Keynote CppCon'23

In a team, you might agree on certain conventions, but that only gets you so
far. Teams change and most of the time, engineers are under pressure to deliver
a feature.  They don't get all the decisions right when they _create_ new
components and their APIs, and they must be prevented from making faulty
assumptions about APIs others (or they themselves) created.

Furthermore, not only is natural for engineers to change their minds about _how_
they want to achieve things, also customers, directors, and thus product owners
change their minds about what they want—past plans are that: plans of the past.
Under these conditions, you must be able to duck-tape a new solution using a
existing code—again, ideally without running the risk of introducing
contradicting assumptions when composing APIs.

So, in summary, there is no news here, _there are no solutions, there are only
trade-offs._ Of course, you could spin the above argument about types ad
infinitum and—figuratively speaking—insist on using Haskell (or whatever), and
spend the rest of your days discussing mind-boggling abstractions until your
head spins—probably not a winning strategy. What is remarkable to me at least is
that Rust managed to make the right trade-offs insofar as it almost _stormed_
the system engineering space.  For the first time, engineers have a practical
alternative to C[++] for security- and performance-critical applications, while,
e.g., still providing benefits in areas like web engineering, smart contracts,
etc., typically confined to dynamically typed "high-level" languages.

Rust is not the last word—god forbid. But empirically speaking, as ugly as it
is—and it is ugly—, it definitively hit the nail on the head in some ways to be
extremely effective for a wide range of applications.

## Concepts

_Get concrete already ..._



* Language Concepts
  * [Sum types](#sum-types) (Algebraic data types)
  * RAII
  * Iterators
  * Scoping
  * Ownership types
  * Send + Sync
* Static Analysis & Linting
* Documentation

# Sum types

_Together with product types (`struct`s and tuples), sum types allow for
**unambiguous** modeling of application state._

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

## `Option<T>`

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

### Intermezzo: `Option::<&T>::None == null`

This is a quick intermezzo regarding the all-farmous «zero-cost abstractions» in
Rust. In the above example, the call to
[`get`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get)
returns a `Option<&V>`, where `V` is the value type of the hash map. In this
case, and whenever `Option` just wraps a reference, there is no overhead
associated with using an option type at runtime. As a null-pointer is an invalid
reference in any case, the `None`-variant is simply represented as a
`null`-Pointer.

## CLI-Argument parsing: a more elaborate use case

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

