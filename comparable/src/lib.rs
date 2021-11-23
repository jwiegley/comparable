//! The `comparable` crate defines the trait [`Comparable`], along with a derive
//! macro for auto-generating instances of this trait for most data types.
//! Primarily the purpose of this trait is to offer a method,
//! [`Comparable::comparison`], by which two values of any type supporting that
//! trait can yield a summary of the differences between them.
//! 
//! Note that unlike other crates that do data differencing (primarily between
//! scalars and collections), `comparable` has been written primarily with testing
//! in mind. That is, the purpose of generating such change descriptions is to
//! enable writing tests that assert the set of expected changes after some
//! operation between an initial state and the resulting state. This goal also
//! means that some types, like
//! [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html),
//! must be differenced after ordering the keys first, so that the set of changes
//! produced can be made deterministic and thus expressible as a test expectation.
//! 
//! To these ends, the macro [`assert_changes!`] is also provided, taking two
//! values of the same type along with an expected "change description" as
//! returned by `foo.comparison(&bar)`. This function uses the
//! [`pretty_assertions`](https://crates.io/crates/pretty_assertions) crate under
//! the hood so that minute differences within deep structures can be easily seen
//! in the failure output.
//! 
//! # Quickstart
//! 
//! If you want to get started quickly with the [`Comparable`] crate to enhance unit
//! testing, do the following:
//! 
//! 1. Add the `comparable` crate as a dependency, enabling `features = ["derive"]`.
//! 2. Derive the `Comparable` trait on as many structs and enums as needed.
//! 3. Structure your unit tests to follow these three phases:
//!    a. Create the initial state or dataset you intend to test and make a copy
//!       of it.
//!    b. Apply your operations and changes to this state.
//!    c. Use [`assert_changes!`] between the initial state and the resulting state
//!       to assert that whatever happened is exactly what you expected to happen.
//! 
//! The main benefit of this approach over the usual method of "probing" the
//! resulting state -- to ensure it changed as you expected it to-- is that it
//! asserts against the exhaustive set of changes to ensure that no unintended
//! side-effects occurred beyond what you expected to happen. In this way, it is
//! both a positive and a negative test: checking for what you expect to see as
//! well as what you don't expect to see.
//! 
//! # The Comparable trait
//! 
//! The [`Comparable`] trait has two associated types and two methods, one pair
//! corresponding to _value descriptions_ and the other to _value changes_:
//! 
//! ```rust
//! pub trait Comparable {
//!     type Desc: std::cmp::PartialEq + std::fmt::Debug;
//!     fn describe(&self) -> Self::Desc;
//! 
//!     type Change: std::cmp::PartialEq + std::fmt::Debug;
//!     fn comparison(&self, other: &Self) -> comparable::Changed<Self::Change>;
//! }
//! ```
//! 
//! ## Descriptions: the [`Comparable::Desc`] associated type
//! 
//! Value descriptions (the [`Comparable::Desc`] associated type) are needed
//! because value hierarchies can involve many types. Perhaps some of these types
//! implement `PartialEq` and `Debug`, but not all. To work around this
//! limitation, the [`Comparable`] derive macro creates a "mirror" of your data
//! structure with all the same constructors ands field, but using the
//! [`Comparable::Desc`] associated type for each of its contained types.
//! 
//! ```
//! # use comparable_derive::*;
//! #[derive(Comparable)]
//! struct MyStruct {
//!   bar: u32,
//!   baz: u32
//! }
//! ```
//! 
//! This generates a description that mirrors the original type, but using type
//! descriptions rather than the types themselves:
//! 
//! ```
//! struct MyStructDesc {
//!   bar: <u32 as comparable::Comparable>::Desc,
//!   baz: <u32 as comparable::Comparable>::Desc
//! }
//! ```
//! 
//! You may also choose an alternate description type, such as a reduced form of a
//! value or some other type entirely. For example, complex structures could
//! describe themselves by the set of changes they represent from a `Default`
//! value. This is so common, that it's supported via a `compare_default` macro
//! attribute provided by `comparable`:
//! 
//! ```
//! # use comparable_derive::*;
//! #[derive(Comparable)]
//! #[compare_default]
//! struct MyStruct { /* ...lots of fields... */ }
//! 
//! impl Default for MyStruct {
//!     fn default() -> Self { MyStruct {} }
//! }
//! ```
//! 
//! For scalars, the [`Comparable::Desc`] type is the same as the type it's
//! describing, and these are called "self-describing".
//! 
//! There are other macro attributes provided for customizing things even further,
//! which are covered below, beginning at the section on [Structures](#structs).
//! 
//! ## Changes: the [`Comparable::Change`] associated type
//! 
//! When two values of a type differ, this difference gets represented using the
//! associated type [`Comparable::Change`]. Such values are produced by the
//! [`Comparable::comparison`] method, which actually returns `Changed<Change>`
//! since the result may be either `Changed::Unchanged` or
//! `Changed::Changed(_changes_)`.[^option]
//! 
//! [^option] `Changed` is just a different flavor of the `Option` type, created
//! to make changesets clearer than just seeing `Some` in various places.
//! 
//! The primary purpose of a [`Comparable::Change`] value is to compare it to a
//! set of changes you expected to see, so design choices have been made to
//! optimize for clarity and printing rather than, say, the ability to transform
//! one value into another by applying a changeset. This is entirely possible give
//! a dataset and a change description, but no work has been done to achieve this
//! goal.
//! 
//! How changes are represented can differ greatly between scalars, collections,
//! structs and enums, so more detail is given below in the section discussing
//! each of these types.
//! 
//! # Scalars
//! 
//! [`Comparable`] traits have been implemented for all of the basic scalar types.
//! These are self-describing, and use a [`Comparable::Change`] structure named
//! after the type that holds the previous and changed values. For example, the
//! following assertions hold:
//! 
//! ```
//! # use comparable::*;
//! assert_changes!(&100, &100, Changed::Unchanged);
//! assert_changes!(&100, &200, Changed::Changed(I32Change(100, 200)));
//! assert_changes!(&true, &false, Changed::Changed(BoolChange(true, false)));
//! assert_changes!(
//!     &"foo",
//!     &"bar",
//!     Changed::Changed(StringChange("foo".to_string(), "bar".to_string())),
//! );
//! ```
//! 
//! # Vec and Set Collections
//! 
//! The set collections for which [`Comparable`] has been implemented are: `Vec`,
//! `HashSet`, and `BTreeSet`.
//! 
//! The `Vec` uses `Vec<VecChange>` to report all of the indices at which changes
//! happened. Note that it cannot detect insertions in the middle, and so will
//! likely report every item as changed from there until the end of the vector, at
//! which point it will report an added member.
//! 
//! `HashSet` and `BTreeSet` types both report changes the same way, using the
//! `SetChange` type. Note that in order for `HashSet` change results to be
//! deterministic, the values in a `HashSet` must support the `Ord` trait so they
//! can be sorted prior to comparison. Sets cannot tell when specific members have
//! change, and so only report changes in terms of `SetChange::Added` and
//! `SetChange::Removed`.
//! 
//! Here are a few examples, taken from the `comparable_test` test suite:
//! 
//! ```
//! # use comparable::*;
//! # use std::collections::HashSet;
//! // Vectors
//! assert_changes!(
//!     &vec![1 as i32, 2],
//!     &vec![1 as i32, 2, 3],
//!     Changed::Changed(vec![VecChange::Added(2, 3)]),
//! );
//! assert_changes!(
//!     &vec![1 as i32, 3],
//!     &vec![1 as i32, 2, 3],
//!     Changed::Changed(vec![
//!         VecChange::Changed(1, I32Change(3, 2)),
//!         VecChange::Added(2, 3),
//!     ]),
//! );
//! assert_changes!(
//!     &vec![1 as i32, 2, 3],
//!     &vec![1 as i32, 3],
//!     Changed::Changed(vec![
//!         VecChange::Changed(1, I32Change(2, 3)),
//!         VecChange::Removed(2, 3),
//!     ]),
//! );
//! assert_changes!(
//!     &vec![1 as i32, 2, 3],
//!     &vec![1 as i32, 4, 3],
//!     Changed::Changed(vec![VecChange::Changed(1, I32Change(2, 4))]),
//! );
//! 
//! // Sets
//! assert_changes!(
//!     &vec![1 as i32, 2].into_iter().collect::<HashSet<_>>(),
//!     &vec![1 as i32, 2, 3].into_iter().collect::<HashSet<_>>(),
//!     Changed::Changed(vec![SetChange::Added(3)]),
//! );
//! assert_changes!(
//!     &vec![1 as i32, 3].into_iter().collect::<HashSet<_>>(),
//!     &vec![1 as i32, 2, 3].into_iter().collect::<HashSet<_>>(),
//!     Changed::Changed(vec![SetChange::Added(2)]),
//! );
//! assert_changes!(
//!     &vec![1 as i32, 2, 3].into_iter().collect::<HashSet<_>>(),
//!     &vec![1 as i32, 3].into_iter().collect::<HashSet<_>>(),
//!     Changed::Changed(vec![SetChange::Removed(2)]),
//! );
//! assert_changes!(
//!     &vec![1 as i32, 2, 3].into_iter().collect::<HashSet<_>>(),
//!     &vec![1 as i32, 4, 3].into_iter().collect::<HashSet<_>>(),
//!     Changed::Changed(vec![SetChange::Added(4), SetChange::Removed(2)]),
//! );
//! ```
//! 
//! Note that if the first `VecChange::Change` above had used an index of 1
//! instead of 0, the resulting failure would look something like this:
//! 
//! ```text
//! running 1 test
//! test test_comparable_bar ... FAILED
//! 
//! failures:
//! 
//! ---- test_comparable_bar stdout ----
//! thread 'test_comparable_bar' panicked at 'assertion failed: `(left == right)`
//! 
//! Diff < left / right > :
//!  Changed(
//!      [
//!          Change(
//! <            1,
//! >            0,
//!              I32Change(
//!                  100,
//!                  200,
//!              ),
//!          ),
//!      ],
//!  )
//! 
//! ', /Users/johnw/src/comparable/comparable/src/lib.rs:19:5
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! 
//! 
//! failures:
//!     test_comparable_bar
//! ```
//! 
//! # Map Collections
//! 
//! The map collections for which [`Comparable`] has been implemented are:
//! `HashMap`, and `BTreeMap`.
//! 
//! Both report changes the same way, using the `MapChange` type. Note that in
//! order for `HashMap` change results to be deterministic, the keys in a
//! `HashMap` must support the `Ord` trait so they can be sorted prior to
//! comparison. Changes are reported in terms of `MapChange::Added`,
//! `MapChange::Removed` and `MapChange::Changed`, exactly like `VecChange` above.
//! 
//! Here are a few examples, taken from the `comparable_test` test suite:
//! 
//! ```
//! # use comparable::*;
//! # use std::collections::HashMap;
//! // HashMaps
//! assert_changes!(
//!     &vec![(0, 1 as i32), (1, 2)].into_iter().collect::<HashMap<_, _>>(),
//!     &vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     Changed::Changed(vec![MapChange::Added(2, 3)]),
//! );
//! assert_changes!(
//!     &vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     &vec![(0, 1 as i32), (1, 2)].into_iter().collect::<HashMap<_, _>>(),
//!     Changed::Changed(vec![MapChange::Removed(2)]),
//! );
//! assert_changes!(
//!     &vec![(0, 1 as i32), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     &vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     Changed::Changed(vec![MapChange::Added(1, 2)]),
//! );
//! assert_changes!(
//!     &vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     &vec![(0, 1 as i32), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     Changed::Changed(vec![MapChange::Removed(1)]),
//! );
//! assert_changes!(
//!     &vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     &vec![(0, 1 as i32), (1, 4), (2, 3)].into_iter().collect::<HashMap<_, _>>(),
//!     Changed::Changed(vec![MapChange::Changed(1, I32Change(2, 4))]),
//! );
//! ```
//! 
//! # <a name="structs"></a>Structures
//! 
//! Differencing arbitrary structures was the original motive for creating
//! `comparable`. This is made feasible using a [`Comparable`] derive macro that
//! auto-generates code needed for such comparisons. The purpose of this section
//! is to explain how this macro works, and the various attribute macros that can
//! be used to guide the process. If all else fails, manual trait implementations
//! are always an alternative.
//! 
//! Here is what deriving `Change` for a structure with multiple fields typically
//! produces:
//! 
//! ```
//! # use comparable_derive::*;
//! # use comparable::*;
//! struct MyStruct {
//!   bar: u32,
//!   baz: u32,
//! }
//! 
//! // The following would be generated by `#[derive(Comparable)]`:
//! 
//! #[derive(PartialEq, Debug)]
//! struct MyStructDesc {
//!     bar: <u32 as Comparable>::Desc,
//!     baz: <u32 as Comparable>::Desc,
//! }
//! 
//! #[derive(PartialEq, Debug)]
//! enum MyStructChange {
//!     Bar(<u32 as Comparable>::Change),
//!     Baz(<u32 as Comparable>::Change),
//! }
//! 
//! impl Comparable for MyStruct {
//!     type Desc = MyStructDesc;
//! 
//!     fn describe(&self) -> Self::Desc {
//!         MyStructDesc {
//!             bar: self.bar.describe(),
//!             baz: self.baz.describe(),
//!         }
//!     }
//! 
//!     type Change = Vec<MyStructChange>;
//! 
//!     fn comparison(&self, other: &Self) -> Changed<Self::Change> {
//!         let changes: Self::Change = vec![
//!             self.bar.comparison(&other.bar).map(MyStructChange::Bar),
//!             self.baz.comparison(&other.baz).map(MyStructChange::Baz),
//!         ]
//!             .into_iter()
//!             .flatten()
//!             .collect();
//!         if changes.is_empty() {
//!             Changed::Unchanged
//!         } else {
//!             Changed::Changed(changes)
//!         }
//!     }
//! }
//! ```
//! 
//! For structs with one field or no fields, see the related section below.
//! 
//! ## Field attribute: `comparable_ignore`
//! 
//! The first attribute macro you'll notice that can be applied to individual
//! fields is `#[comparable_ignore]`, which must be used if the type in question
//! cannot be compared for differences.
//! 
//! ## Field attribute: `comparable_synthetic`
//! 
//! The `#[comparable_synthetic { <BINDINGS...> }]` attribute allows you to attach
//! one or more "synthetic properties" to a field, which are then considered in
//! both descriptions and change sets, as if they were actual fields with the
//! computed value. Here is an example:
//! 
//! ```
//! # use comparable_derive::*;
//! #[derive(Comparable)]
//! pub struct Synthetics {
//!     #[comparable_synthetic {
//!         let full_value = |x: &Self| -> u8 { x.ensemble.iter().sum() };
//!     }]
//!     #[comparable_ignore]
//!     pub ensemble: Vec<u8>,
//! }
//! ```
//! 
//! This structure has an `ensemble` field containing a vector of `u8` values.
//! However, in tests we may not care if the vector's contents change, so long as
//! the final sum remains the same. This is done by ignoring the ensemble field so
//! that it's not generated or described at all, while creating a synthetic field
//! _derived from the full object_ that yields the sum.
//! 
//! Note that the syntax for the `comparable_synthetic` attribute is rather
//! specific: a series of simply-named `let` bindings, where the value in each
//! case is a fully typed closure that takes a reference to the object containing
//! the original field (`&Self`), and yields a value of some type for which
//! [`Comparable`] has been implemented or derived.
//! 
//! ## Deriving `Comparable` for structs: the `Desc` type
//! 
//! By default, deriving [`Comparable`] for a structure will create a "mirror" of
//! that structure, with all the same fields, but replacing every type `T` with
//! `<T as Comparable>::Desc`:
//! 
//! ```
//! # use comparable::*;
//! struct MyStructDesc {
//!   bar: <u32 as Comparable>::Desc,
//!   baz: <u32 as Comparable>::Desc
//! }
//! ```
//! 
//! This process can be influenced using several attribute macros.
//! 
//! ### Macro attribute: `self_describing`
//! 
//! If the `self_describing` attribute is used, the [`Comparable::Desc`] type is
//! set to be the type itself, and the [`Comparable::describe`] method return a
//! clone of the value.
//! 
//! Note the following traits are required for self-describing types: `Clone`,
//! `Debug` and `PartialEq`.
//! 
//! ### Macro attribute: `no_description`
//! 
//! If you want no description at all for a type, since you only care about how it
//! has changed and never want to report a description of the value in any other
//! context, then you can use `#[no_description]`. This sets the
//! [`Comparable::Desc`] type to be unit, and the [`Comparable::describe`] method
//! accordingly:
//! 
//! ```ignore
//! type Desc = ();
//! 
//! fn describe(&self) -> Self::Desc {
//!     ()
//! }
//! ```
//! 
//! It is assumed that when this is appropriate, such values will never appear in
//! any change output, so consider a different approach if you see lots of units
//! turning up.
//! 
//! ### Macro attribute: `describe_type` and `describe_body`
//! 
//! You can have more control over description by specifying exactly the text that
//! should appear for the [`Comparable::Desc`] type and the body of the
//! [`Comparable::describe`] function. Basically, for the following definition:
//! 
//! ```ignore
//! # use comparable_derive::*;
//! #[derive(Comparable)]
//! #[describe_type(T)]
//! #[describe_body(B)]
//! struct MyStruct {
//!   bar: u32,
//!   baz: u32
//! }
//! ```
//! 
//! The following is generated:
//! 
//! ```ignore
//! type Desc = T;
//! 
//! fn describe(&self) -> Self::Desc {
//!     B
//! }
//! ```
//! 
//! This also means that the expression argument passed to `describe_body` may
//! reference the `self` parameter. Here is a real-world use case:
//! 
//! ```
//! # use comparable_derive::*;
//! #[cfg_attr(feature = "comparable",
//!            derive(comparable::Comparable),
//!            describe_type(String),
//!            describe_body(self.to_string()))]
//! struct MyStruct {}
//! ```
//! 
//! This same approach could be used to represent large blobs of data by their
//! checksum hash, for example, or large data structures that you don't need to
//! ever display by their Merkle root hash.
//! 
//! ### Macro attribute: `compare_default`
//! 
//! When the `#[compare_default]` attribute macro is used, the
//! [`Comparable::Desc`] type is defined to be the same as the
//! [`Comparable::Change`] type, with the [`Comparable::describe`] method being
//! implemented as a comparison against the value of `Default::default()`:
//! 
//! ```ignore
//! # use comparable::*;
//! impl comparable::Comparable for MyStruct {
//!     type Desc = Self::Change;
//! 
//!     fn describe(&self) -> Self::Desc {
//!         MyStruct::default().comparison(self).unwrap_or_default()
//!     }
//! 
//!     type Change = Vec<MyStructChange>;
//! 
//!     /* ... */
//! }
//! ```
//! 
//! Note that changes for structures are always a vector, since this allows
//! changes to be reported separately for each field. More on this in the
//! following section.
//! 
//! ## Macro attribute: `comparable_public` and `comparable_private`
//! 
//! By default, the auto-generated [`Comparable::Desc`] and [`Comparable::Change`]
//! types have the same visibility as their parent. This may not be appropriate,
//! however, if you want to keep the original data type private but allow
//! exporting of descriptions and change sets. To support this -- and the converse
//! -- you can use `#[comparable_public]` and `#[comparable_private]` to be
//! explicit about the visibility of these generated types.
//! 
//! ### Special case: Unit structs
//! 
//! If a struct has no fields it can never change, and so only a unitary
//! [`Comparable::Desc`] type is generated.
//! 
//! ### Special case: Singleton structs
//! 
//! If a struct has only one fields, whether named or unnamed, it no longer makes
//! sense to use a vector of enum values to record what has changed. In this case
//! the derivation becomes much simpler:
//! 
//! ```
//! # use comparable_derive::*;
//! # use comparable::*;
//! struct MyStruct {
//!   bar: u32,
//! }
//! 
//! // The following would be generated by `#[derive(Comparable)]`:
//! 
//! #[derive(PartialEq, Debug)]
//! struct MyStructDesc {
//!     bar: <u32 as Comparable>::Desc,
//! }
//! 
//! #[derive(PartialEq, Debug)]
//! struct MyStructChange {
//!     bar: <u32 as Comparable>::Change,
//! }
//! 
//! impl Comparable for MyStruct {
//!     type Desc = MyStructDesc;
//! 
//!     fn describe(&self) -> Self::Desc {
//!         MyStructDesc { bar: self.bar.describe() }
//!     }
//! 
//!     type Change = MyStructChange;
//! 
//!     fn comparison(&self, other: &Self) -> Changed<Self::Change> {
//!         self.bar.comparison(&other.bar).map(|x| MyStructChange { bar: x })
//!     }
//! }
//! ```
//! 
//! ## Deriving `Comparable` for structs: the `Change` type
//! 
//! By default for structs, deriving [`Comparable`] creates an `enum` with
//! variants for each field in the `struct`, and it represents changes using a
//! vector of such values. This means that for the following definition:
//! 
//! ```
//! # use comparable_derive::*;
//! #[derive(Comparable)]
//! struct MyStruct {
//!   bar: u32,
//!   baz: u32
//! }
//! ```
//! 
//! The [`Comparable::Change`] type is defined to be `Vec<MyStructChange>`, with
//! `MyStructChange` as follows:
//! 
//! ```ignore
//! #[derive(PartialEq, Debug)]
//! enum MyStructChange {
//!     Bar(<u32 as Comparable>::Change),
//!     Baz(<u32 as Comparable>::Change),
//! }
//! 
//! impl comparable::Comparable for MyStruct {
//!     type Desc = MyStructDesc;
//!     type Change = Vec<MyStructChange>;
//! }
//! ```
//! 
//! Note that if a struct has only one field, there is no reason to specify
//! changes using a vector, since either the struct is unchanged or just that one
//! field has changed. For this reason, singleton structs optimize away the vector
//! and use `type Change = [type]Change` in their [`Comparable`] derivation,
//! rather than `type Change = Vec<[type]Change>` as for multi-field structs.
//! 
//! Here is an abbreviated example of how this looks when asserting changes for a
//! struct with multiple fields:
//! 
//! ```ignore
//! assert_changes!(
//!     &initial_foo, &later_foo,
//!     Changed::Changed(vec![
//!         MyStructChange::Bar(...),
//!         MyStructChange::Baz(...),
//!     ]));
//! ```
//! 
//! If the field hasn't been changed it won't appear in the vector, and each field
//! appears at most once. The reason for taking this approach is that structures
//! with many, many fields can be represented by a small change set if most of the
//! other fields were left untouched.
//! 
//! # <a name="enums"></a>Enumerations
//! 
//! Enumerations are handled quite differently from structures, for the reason
//! that while a `struct` is always a product of fields, an `enum` can be more
//! than a sum of variants -- but also a sum of products.
//! 
//! To unpack that a bit: By "a product of fields", this means that a `struct` is
//! a simple grouping of typed fields, where the same fields are available for
//! _every_ value of such a structure.
//! 
//! Meanwhile an `enum` is a sum, or choice, among variants. However, some of
//! these variants can themselves contain groups of fields, as though there were
//! an unnamed structure embedded in the variant. Consider the following `enum`:
//! 
//! ```
//! # use comparable_derive::*;
//! #[derive(Comparable)]
//! enum MyEnum {
//!     One(bool),
//!     Two { two: Vec<bool>, two_more: u32 },
//!     Three,
//! }
//! ```
//! 
//! Here we see variant that has a variant with no fields (`Three`), one with
//! unnamed fields (`One`), and one with named fields like a usual structure
//! (`Two`). The problem, though, is that these embedded structures are never
//! represented as independent types, so we can't define [`Comparable`] for them
//! and just compute the differences between the enum arguments. Nor can we just
//! create a copy of the field type with a real name and generate [`Comparable`]
//! for it, because not every value is copyable or clonable, and it gets very
//! tricky to auto-generate a new hierarchy built out fields with reference types
//! all the way down...
//! 
//! Instead, the following gets generated, which can end up being a bit verbose,
//! but captures the full nature of any differences:
//! 
//! ```ignore
//! enum MyEnumChange {
//!     BothOne(<bool as comparable::Comparable>::Change),
//!     BothTwo {
//!         two: Changed<<Vec<bool> as comparable::Comparable>::Change>,
//!         two_more: Changed<Baz as comparable::Comparable>::Change
//!     },
//!     BothThree,
//!     Different(
//!         <MyEnum as comparable::Comparable>::Desc,
//!         <MyEnum as comparable::Comparable>::Desc
//!     ),
//! }
//! ```
//! 
//! Note that variants with singleton fields do not use [`Comparable::Change`],
//! since that information is already reflected when the variant is reported as
//! having changed at all using, for example, `BothOne`. In the case of `BothTwo`,
//! each of the field types is wrapped in `Changed` because it's possible that
//! either one or both of the fields may changed.
//! 
//! Below is a full example of what gets derived for the enum above:
//! 
//! ```
//! # use comparable_derive::*;
//! # use comparable::*;
//! enum MyEnum {
//!     One(bool),
//!     Two { two: Vec<bool>, two_more: u32 },
//!     Three,
//! }
//! 
//! // The following would be generated by `#[derive(Comparable)]`:
//! 
//! #[derive(PartialEq, Debug)]
//! enum MyEnumDesc {
//!     One(<bool as Comparable>::Desc),
//!     Two { two: <Vec<bool> as Comparable>::Desc,
//!           two_more: <u32 as Comparable>::Desc },
//!     Three,
//! }
//! 
//! #[derive(PartialEq, Debug)]
//! enum MyEnumChange {
//!     BothOne(<bool as Comparable>::Change),
//!     BothTwo { two: Changed<<Vec<bool> as Comparable>::Change>,
//!               two_more: Changed<<u32 as Comparable>::Change> },
//!     BothThree,
//!     Different(MyEnumDesc, MyEnumDesc),
//! }
//! 
//! impl Comparable for MyEnum {
//!     type Desc = MyEnumDesc;
//! 
//!     fn describe(&self) -> Self::Desc {
//!         match self {
//!             MyEnum::One(x) => MyEnumDesc::One(x.describe()),
//!             MyEnum::Two { two: x, two_more: y } =>
//!                 MyEnumDesc::Two { two: x.describe(),
//!                                   two_more: y.describe() },
//!             MyEnum::Three => MyEnumDesc::Three,
//!         }
//!     }
//! 
//!     type Change = MyEnumChange;
//! 
//!     fn comparison(&self, other: &Self) -> Changed<Self::Change> {
//!         match (self, other) {
//!             (MyEnum::One(x), MyEnum::One(y)) =>
//!                 x.comparison(&y).map(MyEnumChange::BothOne),
//!             (MyEnum::Two { two: x0, two_more: x1 },
//!              MyEnum::Two { two: y0, two_more: y1 }) => {
//!                 let c0 = x0.comparison(&y0);
//!                 let c1 = x1.comparison(&y1);
//!                 if c0.is_unchanged() && c1.is_unchanged() {
//!                     Changed::Unchanged
//!                 } else {
//!                     Changed::Changed(MyEnumChange::BothTwo {
//!                         two: c0, two_more: c1
//!                     })
//!                 }
//!             }
//!             (MyEnum::Three, MyEnum::Three) => Changed::Unchanged,
//!             (_, _) => Changed::Changed(
//!                 MyEnumChange::Different(self.describe(), other.describe()))
//!         }
//!     }
//! }
//! ```
//! 
//! ## Deriving `Comparable` for enums: the `Desc` type
//! 
//! By default for enums, deriving [`Comparable`] creates a "mirror" of that
//! structure, with all the same variants and fields, but replacing every type `T`
//! with `<T as Comparable>::Desc`:
//! 
//! ```
//! # use comparable::*;
//! enum MyEnumDesc {
//!   Bar(<u32 as Comparable>::Desc),
//!   Baz { some_field: <u32 as Comparable>::Desc }
//! }
//! ```
//! 
//! This process can be influenced using the same attribute macros as for structs,
//! with the exception that synthetic properties are not yet supported on fields
//! of enum variants. Use of this attribute in that context is silently ignored at
//! present.
//! 
//! **TODO**: jww (2021-11-01): Allow for synthetic fields in enum variants.
//! 
//! ## Deriving `Comparable` for enums: the `Change` type
//! 
//! By default for enums, deriving [`Comparable`] create a related `enum` where
//! each variant from the original is represented by a `Both<Name>` variant in the
//! `Change` type, and a new variant named `Different` is added that takes two
//! description of the original enum.
//! 
//! Whenever two enum values are compared and they have different variants, the
//! `Different` variant of the `Change` type is used to represent a description of
//! the differing values. If the values share the same variant, then
//! `Both<Variant>` is used.
//! 
//! Note that `Both<Variant>` has two forms: For variant with a single named or
//! unnamed field, it is simply the `Change` type associated with the original
//! field type; for variants with multiple named or unnamed fields, each `Change`
//! type is also wrapped in a `Changed` structure, to reflect whether that field
//! of the variant changed or not.
//! 
//! ## Field attribute: `variant_struct_fields`
//! 
//! Note that it is possible to treat variant fields as though they were structs,
//! and then to compare them exactly the same way as for structs above. This is
//! not the default because enum variants with named fields typically contain
//! fewer fields on average than structs, and it would increase verbosity in the
//! change description to always have to name these implied structs. However, in
//! cases where the number of fields found in variants is large, it can be just as
//! benifical as for structs.
//! 
//! For this reason, the macro attribute `variant_struct_fields` is provided to
//! derive such transformations. For example, it would cause the following code to
//! be generated, with the main difference between the new `MyEnumTwoChange` type
//! and how it is used:
//! 
//! ```
//! # use comparable_derive::*;
//! # use comparable::*;
//! enum MyEnum {
//!     One(bool),
//!     Two { two: Vec<bool>, two_more: u32 },
//!     Three,
//! }
//! 
//! // The following would be generated by `#[derive(Comparable)]`:
//! 
//! #[derive(PartialEq, Debug)]
//! enum MyEnumDesc {
//!     One(<bool as Comparable>::Desc),
//!     Two { two: <Vec<bool> as Comparable>::Desc,
//!           two_more: <u32 as Comparable>::Desc },
//!     Three,
//! }
//! 
//! #[derive(PartialEq, Debug)]
//! enum MyEnumChange {
//!     BothOne(<bool as Comparable>::Change),
//!     BothTwo(Vec<MyEnumTwoChange>),
//!     BothThree,
//!     Different(MyEnumDesc, MyEnumDesc),
//! }
//! 
//! #[derive(PartialEq, Debug)]
//! enum MyEnumTwoChange {
//!     Two(<Vec<bool> as Comparable>::Change),
//!     TwoMore(<u32 as Comparable>::Change),
//! }
//! 
//! impl Comparable for MyEnum {
//!     type Desc = MyEnumDesc;
//! 
//!     fn describe(&self) -> Self::Desc {
//!         match self {
//!             MyEnum::One(x) => MyEnumDesc::One(x.describe()),
//!             MyEnum::Two { two: x, two_more: y } =>
//!                 MyEnumDesc::Two { two: x.describe(),
//!                                   two_more: y.describe() },
//!             MyEnum::Three => MyEnumDesc::Three,
//!         }
//!     }
//! 
//!     type Change = MyEnumChange;
//! 
//!     fn comparison(&self, other: &Self) -> Changed<Self::Change> {
//!         match (self, other) {
//!             (MyEnum::One(x), MyEnum::One(y)) =>
//!                 x.comparison(&y).map(MyEnumChange::BothOne),
//!             (MyEnum::Two { two: x0, two_more: x1 },
//!              MyEnum::Two { two: y0, two_more: y1 }) => {
//!                 let c0 = x0.comparison(&y0);
//!                 let c1 = x1.comparison(&y1);
//!                 let changes: Vec<MyEnumTwoChange> = vec![
//!                     c0.map(MyEnumTwoChange::Two),
//!                     c1.map(MyEnumTwoChange::TwoMore),
//!                 ].into_iter().flatten().collect();
//!                 if changes.is_empty() {
//!                     Changed::Unchanged
//!                 } else {
//!                     Changed::Changed(MyEnumChange::BothTwo(changes))
//!                 }
//!             }
//!             (MyEnum::Three, MyEnum::Three) => Changed::Unchanged,
//!             (_, _) => Changed::Changed(
//!                 MyEnumChange::Different(self.describe(), other.describe()))
//!         }
//!     }
//! }
//! ```
//! 
//! ## Special case: Empty enums
//! 
//! If a enum has no variants it cannot be constructed, so both the
//! [`Comparable::Desc`] or [`Comparable::Change`] types are omitted and it is
//! always reported as unchanged.
//! 
//! # <a name="unions"></a>Unions
//! 
//! Unions cannot derive [`Comparable`] instances at the present time.
#[doc(hidden)]
pub mod boxes;
#[doc(hidden)]
pub mod empty;
#[doc(hidden)]
pub mod map;
#[doc(hidden)]
pub mod opt;
#[doc(hidden)]
pub mod scalar;
#[doc(hidden)]
pub mod set;
#[doc(hidden)]
pub mod string;
#[doc(hidden)]
pub mod types;

#[doc(hidden)]
pub use crate::boxes::*;
#[doc(hidden)]
pub use crate::map::*;
#[doc(hidden)]
pub use crate::opt::*;
#[doc(hidden)]
pub use crate::scalar::*;
#[doc(hidden)]
pub use crate::set::*;
#[doc(hidden)]
pub use crate::string::*;

pub use crate::types::{Changed, Comparable};

/// Assert that two values of a type have changed in the way described.
#[macro_export]
macro_rules! assert_changes {
    ($left:expr, $right:expr, $expected:expr$(,)?) => {{
        use comparable::Comparable;
        assert_eq!($expected, $left.comparison(&$right));
    }};
}

/// Assert that two values of a type have either not changed, or they have
/// changed only to the extent described by the give change set. This allows
/// tests to confirm that what they expected to see happened, and anything
/// they didn't expect to see in fact did not happen.
///
/// This function is just a wrapper around `pretty_assertions::assert_eq!`
/// and does the following:
/// ```ignore
/// pretty_assertions::assert_eq!(expected, left.comparison(right))
/// ```
#[macro_export]
macro_rules! pretty_assert_changes {
    ($left:expr, $right:expr, $expected:expr$(,)?) => {{
        use comparable::Comparable;
        pretty_assertions::assert_eq!($expected, $left.comparison(&$right));
    }};
}

/// Assert that two values of a type have changed in the way described,
/// stopping at the first failure if this occurs in a proptest block.
#[macro_export]
macro_rules! prop_assert_changes {
    ($left:expr, $right:expr, $expected:expr$(,)?) => {{
        use comparable::Comparable;
        proptest::prop_assert_eq!($expected, $left.comparison(&$right));
    }};
}

/// Assert that two values of a type have changed in the way described,
/// stopping at the first failure if this occurs in a proptest block.
#[macro_export]
macro_rules! prop_pretty_assert_changes {
    ($left:expr, $right:expr, $expected:expr$(,)?) => {{
        use comparable::Comparable;
        let changes = $left.comparison(&$right);
        if $expected != changes {
            return Err(proptest::test_runner::TestCaseError::fail(format!(
                "prop_assert_changes! failed: `(left == right)`\
                 \n\
                 \n{}\
                 \n",
                pretty_assertions::Comparison::new(&$expected, &changes)
            )));
        }
    }};
}

// Re-export #[derive(Comparable)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "comparable_derive")]
#[doc(hidden)]
pub use comparable_derive::*;
