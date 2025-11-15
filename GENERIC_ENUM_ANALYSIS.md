# Analysis: Generic Type Support in #[derive(Comparable)]

## Summary

The `#[derive(Comparable)]` macro does not currently support generic types (enums or structs with type parameters). The macro generates `Desc` and `Change` types that lose the generic parameters from the original type definition.

## Reproduction

### Test Case

Created test file: `/Users/johnw/src/comparable/work/fix-10/comparable_test/test/generic_enum_test.rs`

```rust
use std::fmt::Debug;
use comparable::Comparable;

#[derive(Debug, PartialEq, Comparable)]
pub enum IdOrObject<T: PartialEq + Debug> {
    ID(String),
    Object(T),
}
```

### Compilation Errors

```
error[E0412]: cannot find type `T` in this scope
 --> comparable_test/test/generic_enum_test.rs:7:12
  |
7 |     Object(T),
  |            ^ not found in this scope

error[E0107]: missing generics for enum `IdOrObject`
 --> comparable_test/test/generic_enum_test.rs:5:10
  |
5 | pub enum IdOrObject<T: PartialEq + Debug> {
  |          ^^^^^^^^^^ expected 1 generic argument

error[E0282]: type annotations needed
 --> comparable_test/test/generic_enum_test.rs:4:28
  |
4 | #[derive(Debug, PartialEq, Comparable)]
  |                            ^^^^^^^^^^ cannot infer type
```

## Root Cause Analysis

### What the Macro Generates (via cargo expand)

For the input enum:
```rust
pub enum IdOrObject<T: PartialEq + Debug> {
    ID(String),
    Object(T),
}
```

The macro generates:

```rust
// GENERATED - MISSING GENERIC PARAMETER!
pub enum IdOrObjectDesc {
    ID(<String as comparable::Comparable>::Desc),
    Object(<T as comparable::Comparable>::Desc),  // ERROR: T is undefined!
}

// GENERATED - MISSING GENERIC PARAMETER!
pub enum IdOrObjectChange {
    BothID(<String as comparable::Comparable>::Change),
    BothObject(<T as comparable::Comparable>::Change),  // ERROR: T is undefined!
    Different(
        <IdOrObject as comparable::Comparable>::Desc,  // ERROR: IdOrObject needs type param
        <IdOrObject as comparable::Comparable>::Desc,
    ),
}

// GENERATED - MISSING GENERIC PARAMETER!
impl comparable::Comparable for IdOrObject {  // ERROR: IdOrObject needs type param
    type Desc = IdOrObjectDesc;  // ERROR: IdOrObjectDesc needs type param
    // ... rest of impl
}
```

### What Should Be Generated

```rust
// CORRECT - with generic parameters and bounds
pub enum IdOrObjectDesc<T: PartialEq + Debug> {
    ID(<String as comparable::Comparable>::Desc),
    Object(<T as comparable::Comparable>::Desc),
}

pub enum IdOrObjectChange<T: PartialEq + Debug> {
    BothID(<String as comparable::Comparable>::Change),
    BothObject(<T as comparable::Comparable>::Change),
    Different(
        <IdOrObject<T> as comparable::Comparable>::Desc,
        <IdOrObject<T> as comparable::Comparable>::Desc,
    ),
}

impl<T: PartialEq + Debug> comparable::Comparable for IdOrObject<T>
where
    T: comparable::Comparable,  // Need this bound!
{
    type Desc = IdOrObjectDesc<T>;
    type Change = IdOrObjectChange<T>;
    // ... rest of impl
}
```

## Code Locations and Issues

### 1. Type Definition Generation (`comparable_derive/src/utils.rs:186-273`)

**Current Code (line 271):**
```rust
pub fn generate_type_definition(visibility: &syn::Visibility, type_name: &syn::Ident, data: &syn::Data) -> TokenStream {
    // ... processes fields ...
    quote! {
        #derive_serde
        #[derive(PartialEq, Debug)]
        #visibility #keyword #type_name#body  // MISSING: generics!
    }
}
```

**Problem:** The function signature only takes `type_name: &syn::Ident`, losing all generic parameter information from the original `DeriveInput`.

**What's Missing:**
- Generic parameters (`<T>`)
- Where clauses
- Generic bounds (`: PartialEq + Debug`)

### 2. Input Processing (`comparable_derive/src/inputs.rs`)

**Current Code:**
```rust
pub struct Inputs<'a> {
    pub attrs: Attributes,
    pub input: &'a syn::DeriveInput,  // Has generics!
    pub visibility: syn::Visibility,
}
```

**Note:** The `input` field contains the full `DeriveInput` which includes `input.generics`, but this information is never passed to `generate_type_definition`.

### 3. Desc/Change Type Generation (`comparable_derive/src/definition.rs`)

**Line 50-59 (Desc type):**
```rust
pub fn generate_desc_type(inputs: &Inputs) -> Self {
    let type_name = &inputs.input.ident;
    let desc_name = format_ident!("{}{}", &inputs.input.ident, inputs.attrs.comparable_desc_suffix);
    let desc_type = generate_type_definition(
        &inputs.visibility,
        &desc_name,
        &map_on_fields_over_data(true, &inputs.input.data, |r| syn::Field {
            ty: Self::assoc_type(&r.field.ty, "Desc"),
            ..r.field.clone()
        }),
    );
    // ... rest
}
```

**Problem:** Only passes `desc_name` (an `Ident`), not the full generics.

**Line 123-136 (Change type):** Same issue.

### 4. Trait Implementation (`comparable_derive/src/outputs.rs:37-57`)

**Current Code:**
```rust
fn impl_comparable(
    name: &syn::Ident,  // Just the name!
    describe_type: &syn::Type,
    describe_body: &TokenStream,
    change_type: &syn::Type,
    change_body: &TokenStream,
) -> TokenStream {
    quote! {
        impl comparable::Comparable for #name {  // MISSING: impl<T> for Type<T>
            type Desc = #describe_type;
            // ...
        }
    }
}
```

**Problem:** No generic parameters in the impl block.

## Required Changes

### High-Level Strategy

1. **Extract generics from input:** Use `inputs.input.generics` which is of type `syn::Generics`
2. **Propagate generics through the call chain:** Thread generics through all generation functions
3. **Add generics to generated types:** Include `<T, U, ...>` after type names
4. **Add bounds to trait impl:** Include `impl<T: Bounds> Comparable for Type<T> where T: Comparable`
5. **Handle where clauses:** Preserve and extend where clauses from original type

### Detailed Changes Needed

#### 1. Update `generate_type_definition` signature

**File:** `comparable_derive/src/utils.rs:186`

**Current:**
```rust
pub fn generate_type_definition(
    visibility: &syn::Visibility,
    type_name: &syn::Ident,
    data: &syn::Data
) -> TokenStream
```

**Needed:**
```rust
pub fn generate_type_definition(
    visibility: &syn::Visibility,
    type_name: &syn::Ident,
    generics: &syn::Generics,  // ADD THIS
    data: &syn::Data
) -> TokenStream
```

**Implementation change (line 268-273):**
```rust
// Extract generic params and where clause
let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

quote! {
    #derive_serde
    #[derive(PartialEq, Debug)]
    #visibility #keyword #type_name#ty_generics #body #where_clause  // UPDATED
}
```

#### 2. Add generic bounds to generated types

The generated `Desc` and `Change` types need to ensure that all type parameters implement `Comparable`:

```rust
// For IdOrObject<T>, we need:
impl<T: PartialEq + Debug> comparable::Comparable for IdOrObject<T>
where
    T: comparable::Comparable,  // Additional bound!
{
    // ...
}
```

This requires:
1. Clone the generics from the original type
2. Add `T: comparable::Comparable` bounds for each type parameter
3. Merge with existing where clauses

**Implementation approach:**
```rust
// In definition.rs or outputs.rs
fn add_comparable_bounds(generics: &syn::Generics) -> syn::Generics {
    let mut new_generics = generics.clone();

    // For each generic type parameter, add `: Comparable` bound
    for param in &mut new_generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param.bounds.push(syn::parse_quote!(comparable::Comparable));
        }
    }

    new_generics
}
```

#### 3. Update `impl_comparable` signature

**File:** `comparable_derive/src/outputs.rs:37`

**Current:**
```rust
fn impl_comparable(
    name: &syn::Ident,
    describe_type: &syn::Type,
    describe_body: &TokenStream,
    change_type: &syn::Type,
    change_body: &TokenStream,
) -> TokenStream
```

**Needed:**
```rust
fn impl_comparable(
    name: &syn::Ident,
    generics: &syn::Generics,  // ADD THIS
    describe_type: &syn::Type,
    describe_body: &TokenStream,
    change_type: &syn::Type,
    change_body: &TokenStream,
) -> TokenStream
```

**Implementation (line 44-56):**
```rust
let generics_with_bounds = add_comparable_bounds(generics);
let (impl_generics, ty_generics, where_clause) = generics_with_bounds.split_for_impl();

quote! {
    impl #impl_generics comparable::Comparable for #name #ty_generics #where_clause {
        type Desc = #describe_type;
        fn describe(&self) -> Self::Desc {
            #describe_body
        }

        type Change = #change_type;
        fn comparison(&self, other: &Self) -> comparable::Changed<Self::Change> {
            #change_body
        }
    }
}
```

#### 4. Update all call sites

Need to thread `generics` through:
- `definition.rs:50-59` (generate_desc_type)
- `definition.rs:123-136` (generate_change_type)
- `outputs.rs:17` (impl_comparable call)
- All callers of `generate_type_definition`

#### 5. Handle type references in generated code

When referencing the generated types, need to include type parameters:

**Current (in enums.rs:106):**
```rust
ty: Definition::assoc_type(&ident_to_type(type_name), "Desc"),
```

**Needed:**
```rust
// Build type with generics: IdOrObject<T>
let type_with_generics = /* ... construct from type_name + generics ... */;
ty: Definition::assoc_type(&type_with_generics, "Desc"),
```

## Testing Strategy

### Test Cases to Add

1. **Generic enum (single type param):**
   ```rust
   #[derive(Comparable)]
   enum IdOrObject<T: Debug> {
       ID(String),
       Object(T),
   }
   ```

2. **Generic enum (multiple type params):**
   ```rust
   #[derive(Comparable)]
   enum Either<L, R> {
       Left(L),
       Right(R),
   }
   ```

3. **Generic struct:**
   ```rust
   #[derive(Comparable)]
   struct Container<T> {
       value: T,
   }
   ```

4. **Generic with where clause:**
   ```rust
   #[derive(Comparable)]
   enum MyEnum<T>
   where
       T: Clone + Debug,
   {
       Value(T),
   }
   ```

5. **Generic with lifetime parameters:**
   ```rust
   #[derive(Comparable)]
   struct Wrapper<'a, T> {
       data: &'a T,
   }
   ```

6. **Nested generics:**
   ```rust
   #[derive(Comparable)]
   struct Nested<T> {
       inner: Option<T>,
   }
   ```

## Complexity Assessment

**Difficulty: Medium-High**

**Challenges:**
1. Generic parameters need to be propagated through ~5 layers of function calls
2. Need to correctly handle:
   - Type parameters (`<T>`)
   - Lifetime parameters (`<'a>`)
   - Const parameters (`<const N: usize>`)
   - Where clauses
   - Trait bounds
3. Generated types must reference the generic parameters correctly
4. The `Comparable` bound must be added without conflicting with user bounds
5. Backwards compatibility must be maintained for non-generic types

**Estimated Changes:**
- ~10 files to modify
- ~30-50 lines of new/changed code
- Extensive testing needed

## Alternative Approaches

### Option 1: Full Generic Support (Recommended)

Implement complete generic support as described above.

**Pros:**
- Most correct and flexible solution
- Handles all generic scenarios
- Consistent with Rust's derive system

**Cons:**
- More complex implementation
- Needs careful testing

### Option 2: Limited Generic Support

Only support type parameters without bounds or where clauses.

**Pros:**
- Simpler implementation
- Covers common cases

**Cons:**
- Doesn't handle complex generic scenarios
- Inconsistent behavior

### Option 3: Better Error Messages

Keep the limitation but provide clear compile-time errors.

**Pros:**
- Very simple to implement
- Doesn't break existing code

**Cons:**
- Doesn't solve the user's problem
- Limits library usefulness

## References

- `syn::Generics` documentation: https://docs.rs/syn/latest/syn/struct.Generics.html
- `syn::Generics::split_for_impl`: https://docs.rs/syn/latest/syn/struct.Generics.html#method.split_for_impl
- Similar issue in serde: https://github.com/serde-rs/serde/blob/master/serde_derive/src/bound.rs

## Next Steps

1. Confirm approach with maintainer
2. Start with simplest case (single type param, no bounds)
3. Incrementally add support for more complex scenarios
4. Add comprehensive test coverage
5. Update documentation
