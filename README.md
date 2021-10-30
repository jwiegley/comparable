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
between an initial state and the resulting state.

To this end, the function `delta::assert_changes` is also provided, taking two
values of the same type along with an expected "change description" as
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

### Value descriptions

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
which are covered below in the section on [Structs and
enums](#structs_and_enums).

### Value changes

When two values of a type are different, this difference is captured using the
associated type `Change`. Such values are produced by the `delta` method,
which actually returns `Changed<Change>` since the result may be either
`Changed::Unchanged` or `Changed::Changed(_changes_)`.

The primary purpose of a `Change` value is to compare it to a set of changes
you expected to see.

## Scalars

## Collections

## <a name="structs_and_enums"></a>Structs and enums
