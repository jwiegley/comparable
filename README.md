# Delta: Structural differencing in Rust

The `delta` crate defines the trait `Delta`, along with a derive macro for
auto-generating instances of this trait for most data types. Primarily the
purpose of this trait is to offer a method, `delta`, by which two values of
any type supporting that trait can yield a summary of the differences between
them.

Note that unlike other crates that do data differencing (primarily between
scalars and collections), `delta` has been written primarily with testing in
mind. That is, the purpose of generating such change descriptions is to enable
writing tests that assert the set of expected changes after some operation
between an initial state and the resulting state. This goal also means that
some types, like `HashMap`, must be differenced after ordering the keys first,
so that the set of changes produced can be made deterministic and thus
expressible as a test expectation.

To these ends, the function `delta::assert_changes` is also provided, taking
two values of the same type along with an expected "change description" as
returned by `foo.delta(&bar)`. This function uses the `pretty_assertions`
crate under the hood so that minute differences within deep structures can be
easily seen in the failure output.

## Quickstart

If you want to get started quickly with the `delta` crate to enhance unit
testing, do the following:

1. Add the `delta` crate as a dependency, enabling `features = ["derive"]`.
2. Derive the `delta::Delta` trait on as many structs and enums as needed.
3. Structure your unit tests to follow these three phases:
   a. Create the initial state or dataset you intend to test and make a copy
      of it.
   b. Apply your operations and changes to this state.
   c. Use `delta::assert_changes` between the initial state and the resulting
      state to assert that whatever happened is exactly what you expected to
      happen.

The main benefit of this approach over the usual method of "probing" the
resulting state -- to ensure it changed as you expected it to-- is that it
asserts against the exhaustive set of changes to ensure that no unintended
side-effects occurred beyond what you expected to happen. In this way, it is
both a positive and a negative test: checking for what you expect to see as
well as what you don't expect to see.

## The Delta trait

The `Delta` trait has two associated types and two methods, one pair
corresponding to _value descriptions_ and the other to _value changes_:

```rust
pub trait Delta {
    type Desc: PartialEq + Debug;
    fn describe(&self) -> Self::Desc;

    type Change: PartialEq + Debug;
    fn delta(&self, other: &Self) -> Changed<Self::Change>;
}
```

### Descriptions: the `Desc` associated type

The reason for value descriptions (`Desc`) is that not every value can or
should be directly represented in a change report. For example, if two values
were added to a `Vec`, it would be reported in a change set as follows:

```rust
vec![ VecChange::Added(_description_),
      VecChange::Added(_description_)
    ]
```

For scalars, it's normal for the `Desc` type to be the same as the type it's
describing, and these are called "self-describing". But not every type is able
to implement the `PartialEq` and `Debug` traits needed by `Desc` so that
change sets can be compared and displayed. For this reason, trait implementors
are able to define an alternate description type. This can be a reduced form
of the value, a projection, or some other value entirely.

For example, complex structures could describe themselves by the set of
changes they represent from a `Default` value. This is so common, in fact,
that it's supported using a `compare_default` macro attribute provided by
`delta`:

```rust
#[derive(Delta)]
#[compare_default]
struct Foo { /* ...lots of fields... */ }

impl Default for Foo { /* ... */ }
```

Another reason why separate `Desc` types are often needed is that value
hierarchies may involve many types. Perhaps some of these support equality and
printing, but not all. Thus, if you use the `Delta` derive macro on structure
or enum types, a "mirror" of the data structure is created with the same
constructors ands field, but using the `Desc` associated type for each of its
contained types. For example:

```rust
#[derive(Delta)]
struct Foo {
  bar: Bar,
  baz: Baz
}
```

This generates a description that mirrors the original type, but using type
descriptions rather than the types themselves:

```rust
struct FooDesc {
  bar: <Bar as Delta>::Desc,
  baz: <Baz as Delta>::Desc
}
```

There are other macro attributes provided for customizing things even further,
which are covered below, beginning at the section on [Structures](#structs).

### Changes: the `Change` associated type

When two values of a type differ, this difference gets represented using the
associated type `Change`. Such values are produced by the `delta` method,
which actually returns `Changed<Change>` since the result may be either
`Changed::Unchanged` or `Changed::Changed(_changes_)`.[^option]

[^option] `Changed` is just a different flavor of the `Option` type, created
to make changesets clearer than just seeing `Some` in various places.

The primary purpose of a `Change` value is to compare it to a set of changes
you expected to see, so design choices have been made to optimize for clarity
and printing rather than, say, the ability to transform one value into another
by applying a changeset. This is entirely possible give a dataset and a change
description, but no work has been done to achieve this goal.

How changes are represented can differ greatly between scalars, collections,
structs and enums, so more detail is given below in the section discussing
each of these types.

## Scalars

`Delta` traits have been implemented for all of the basic scalar types. These
are self-describing, and use a `Change` structure named after the type that
holds the previous and changed values. For example, the following assertions
hold:

```rust
assert_changes(&100, &100, Changed::Unchanged);
assert_changes(&100, &200, Changed::Changed(I32Change(100, 200)));
assert_changes(&true, &false, Changed::Changed(BoolChange(true, false)));
assert_changes(
    &"foo",
    &"bar",
    Changed::Changed(StringChange("foo".to_string(), "bar".to_string())),
);
```

## Collections

The set of collections for which `Delta` has been implemented are: `Vec`,
`HashSet`, `BTreeSet`, `HashMap` and `HashSet`.

The `Vec`, `HashSet` and `BTreeSet` types all report changes the same way,
using the `SetChange` type. Further, in order for `HashSet` change results to
be deterministic, the values in a `HashSet` must also support the `Ord` trait
so they can be sorted prior to comparison.

Some examples follow, using `Vec`. Note that `HashSet` and `BTreeSet` are
similar, but use a `SetChange` structure that has no `Change` constructor
since we don't know which values have been changed, only if they have been
added or removed.

```rust
assert_changes(&vec![100], &vec![100], Changed::Unchanged);
assert_changes(
    &vec![100],
    &vec![200],
    Changed::Changed(vec![VecChange::Change(0, I32Change(100, 200))]),
);
assert_changes(
    &vec![],
    &vec![100],
    Changed::Changed(vec![VecChange::Added(100)]),
);
assert_changes(
    &vec![100],
    &vec![],
    Changed::Changed(vec![VecChange::Removed(100)]),
);
assert_changes(
    &vec![100, 200, 300],
    &vec![100, 400, 300],
    Changed::Changed(vec![VecChange::Change(1, I32Change(200, 400))]),
);
assert_changes(
    &vec![100, 200, 300],
    &vec![100, 400, 300],
    Changed::Changed(vec![VecChange::Change(1, I32Change(200, 400))]),
);

// The same as the last example, but use `HashSet` instead of `Vec`
assert_changes(
    &HashSet::from(vec![100, 200, 300].into_iter().collect()),
    &HashSet::from(vec![100, 400, 300].into_iter().collect()),
    Changed::Changed(vec![SetChange::Added(400), SetChange::Removed(200)]),
);
```

Note that if the first `VecChange::Change` above had used an index of 1
instead of 0, the resulting failure would look something like this:

```
running 1 test
test test_delta_bar ... FAILED

failures:

---- test_delta_bar stdout ----
thread 'test_delta_bar' panicked at 'assertion failed: `(left == right)`

Diff < left / right > :
 Changed(
     [
         Change(
<            1,
>            0,
             I32Change(
                 100,
                 200,
             ),
         ),
     ],
 )

', /Users/johnw/src/delta/delta/src/lib.rs:19:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_delta_bar
```

## <a name="structs"></a>Structures

Differencing arbitrary structures was the original impetus for creating
`delta`. This is made feasible with a `Delta` derive macro that auto-generates
the code needed for such comparisons. The purpose of this section is to
understand how this macro works, and the various attribute macros that can be
used to guide the process. If all else fails, manual trait implementations are
always an alternative.

For the purpose of the following sub-sections, we consider the following
structure:

```rust
#[derive(Delta)]
struct Foo {
  bar: Bar,
  baz: Baz,
  #[delta_ignore]
  baz: Box<dyn FnOnce(u32)>
}
```

The first attribute macro you'll notice that can be applied to individual
fields is `#[delta_ignore]`, which must be used if the type in question cannot
be compared for differences.

**TODO**: Provide an attribute macro `#[delta_wrap]` that defines a wrapping
type that can be used for comparison. When the field is encountered during
`delta`, construct a temporary value using the wrapper and then call `delta`
on that.

**TODO**: Provide an attribute macro `#[delta_view(function)]` for defining
synthetic properties that receive `&self` as an argument and return a type
implementing `Delta` that can be differenced.

### Deriving Delta for structs: the Desc type

By default, deriving `Delta` for a structure will create a "mirror" of that
structure, with all the same fields, but replacing every type `T` with `<T as
Delta>::Desc`:

```rust
struct FooDesc {
  bar: <Bar as Delta>::Desc,
  baz: <Baz as Delta>::Desc
}
```

This process can be influenced using several attribute macros.

#### `compare_default`

When the `#[compare_default]` attribute macro is used, the `Desc` type is
defined to be the same as the `Change` type, with the `describe` method being
implemented as a comparison against the value of `Default::default()`:

```rust
type Desc = Self::Change;

fn describe(&self) -> Self::Desc {
    Foo::default().delta(self).unwrap_or_default()
}

type Change = Vec<FooChange>;
```

Note that changes for structures are always a vector, since this allows
changes to be reported separately for each field. More on this in the
following section.

#### `no_description`

If you want no description at all for a type, since you only care about how it
has changed and never want to report a description of the value in any other
context, then you can use `#[no_description]`. This sets the `Desc` type to be
unit, and the `describe` method accordingly:

```rust
type Desc = ();

fn describe(&self) -> Self::Desc {
    ()
}
```

It is assumed that when this is appropriate, such values will never appear in
any change output, so consider a different approach if you see lots of units
turning up.

#### `describe_type` and `describe_body`

You can have more control over description by specifying exactly the text that
should appear for the `Desc` type and the body of the `describe` function.
Basically, for the following definition:

```rust
#[derive(Delta)]
#[describe_type(T)]
#[describe_body(B)]
struct Foo {
  bar: Bar,
  baz: Baz
}
```

The following code is generated:

```rust
type Desc = T;

fn describe(&self) -> Self::Desc {
    B
}
```

This also means that the expression argument passed to `describe_body` may
reference the `self` parameter. Here is a real-world example:

```rust
#[cfg_attr(feature = "delta",
           derive(delta::Delta),
           describe_type(String),
           describe_body(self.to_string()))]
```

This same approach could be used to represent large blobs of data by their
checksum hash, or large data structures that you don't need displayed by a
Merkle root hash.

### Deriving Delta for structs: the Change type

By default for structs, deriving `Delta` creates an `enum` with variants for
every field in the `struct`, and associated the `Change` type with a vector of
such values. This means that for the following definition:

```rust
#[derive(Delta)]
struct Foo {
  bar: Bar,
  baz: Baz
}
```

The `Change` type is defined to be `Vec<FooChange>`, with `FooChange` defined
as follows:

```rust
#[derive(PartialEq, Debug)]
enum FooChange {
    Bar(<Bar as Delta>::Change),
    Baz(<Baz as Delta>::Change),
}

impl Delta for Foo {
    type Desc = FooDesc;
    type Change = Vec<FooChange>;
}
```

Here is an abbreviated example:

```rust
assert_changes(
    &initial_foo, &later_foo,
    Changed::Changed(vec![
        FooChange::Bar(...),
        FooChange::Baz(...),
    ]));
```

Of course, if the field hasn't changed it won't appear in the vector, and each
field appears at most once. The reason for taking this approach is that
structures with many, many fields can be represented by a very small change
set if most of the other fields have been left untouched.

#### `public_change` and `private_change`

By default, the auto-generated `Desc` and `Change` types have the same
visibility as their parent. This may not be appropriate, though, if you want
to keep the original data type private but allow exporting of descriptions or
change sets. To support this -- and the converse -- you can use
`#[public_change]` and `#[private_change]` to be explicit about the visibility
of the generated `Desc` and `Change` types.

## <a name="enums"></a>Enumerations

Enumerations are handled quite a bit differently from structures, for the main
reason that while a `struct` is always a product of fields, an `enum` can be
more than just a sum of variants, but also a sum of products.

To unpack that a bit: By a product of fields, I mean that a `struct` is a
simple grouping of typed fields, where the same fields are available for
_every_ value of such a structure.

Meanwhile, an `enum` is a sum, or choice, among variants, but some of these
variants can themselves be groups of fields, as though an unnamed structure
had been embedded in the variant. Consider the following `enum`, which will be
used for all the following examples:

```rust
#[derive(Delta)]
enum MyEnum {
    One(bool),
    Two { two: Vec<bool>, two_more: Baz },
    Three(Bar),
    Four,
}
```

Here we see variant that have unit type (`Four`), unnamed fields (`One` and
`Three`), and named fields like a usual structure (`Two`). The problem,
though, is that these embedded structures are never represented as a separate
type, and so we can't define `Delta` for them.

## <a name="unions"></a>Unions
