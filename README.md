# ProtoSpec

## Purpose
ProtoSpec is inspired by Google's ProtoBuf, but attempts to provide a binary format language capable of representing any binary format as opposed to "staying in its lane".

## Status
ProtoSpec is working and minimally tested. Some features are not yet implemented, and the compiler is still a bit messy with leaky/undocumented semantics and panics. This project is a work in progress (and probably always will be of course). Contributions welcome.

## General Design & Terminology

### Type Declaration
A ProtoSpec `type` declaration is the primary top-level declaration for protospec files.
Example:
```
type test = u32;
```

Declared types are encodable and decodable, and can be interpreted as simultaneously a series of function declarations and a type declaration in the target language.

They can have a condition, which if evaluated to `false` will cause the type declaration to encode to an empty byte array.

They can have an arbitrary number of transformations.

#### Arguments
Type declarations may have an arbitrary number of arguments.
Example:
```
type example(compressed: bool) = container {
    len: u32,
    inner: container [len] {
        data: [..]
    } -> gzip {compressed},
};
```

Arguments are specified when encoding top level types by extra parameters in the `encode_*`/`decode_*` functions. When called from another protospec type, they are declared via function-call-like syntax:
```
type example_compressed = example(true);
```

### Const Declaration
A ProtoSpec `const` declaration is an extra top-level declaration for protospec files. It can be used to store relevant, specific constants.
Example:
```
const X: u32 = 1 + 2;
```

Their associated type MUST NOT have any conditions or transformations. They are generally expected to only be primitive types.

### Import Declaration
A ProtoSpec `import` declaration can import types declared in other ProtoSpec files via relative path. Code for the entire imported file will be generated.
Example:
```
import test_container from "test-import";

type test_impl = test_container[2];
```

### Enum
A ProtoSpec `enum` type can only be defined as a top-level type (directly by a type declaration). It is, in essence, the same as a `const` declaration, but can be represented better in the target language in some cases, and can be cleaner to use in some cases. They MUST be backed by a scalar (integer) representation type.
Example:
```
type test = enum i32 {
    west = 1,
    east, // value of 2
    north = 6,
    south, // value of 7
};
```
ProtoSpec does not have tagged unions due to the ambiguity of representation/encoding.

### Container
A ProtoSpec `container` type is the most powerful type in ProtoSpec. It is similar to a struct.
Containers contain adjacently-encoded fields, with each field having its own name, type, condition, and transformations.
Containers can nest other containers. Nested containers cannot have conditions, or be the inner element of an array.

Fields can have any number of flags after their listed type. Currently declared flags include:
* `+auto`: When encoding, any declared value is ignored, and a container length constraint is used to infer the field value.

Example:
```
type test = container {
    len: u32 +auto,
    inner: container [len] {
        tag: u8[32],
        data: u8[..],
    } -> gzip {is_compressed},
};
```

### Array
A ProtoSpec array types are the second most powerful type in ProtoSpec. They may contain any inner element type. The array itself may have transformations and conditions in accordance to its owning/parent type.
*unimplemented* The inner type may contain transformations or conditions.

Arrays may denote a specific length, referencing a prior-declared field, constant values, or some combination thereof.
Example:
```
container {
    ex1: u8[7],
    ex2: u32,
    ex3: u8[ex2],
}
```

Arrays may denote an unbounded array via `[..]` which consume all available data. If an end of stream is encountered within the decoding of the inner type, it is an error.
Example:
```
container {
    example: u8[..],
}
```

Arrays may denote a sequence-terminated unbounded array via `[.."X"]` where X is any string sequence for which to terminate the string.
```
container {
    my_c_string: u8[.."\0"],
    my_next_c_string: u8[.."\0"],
    all_the_strings: u8[.."\0"][..],
}
```

### Foreign Types
Foreign types in ProtoSpec allow implementation-dependent structures that can express things not otherwise possible in ProtoSpec.
Example usage:
```
import_ffi test_type as type;

type example = test_type[2];
```
Example implementation:
* See `./src/prelude/var.rs`

They may include arguments similar to type declarations.

### Condition
Conditions in ProtoSpec are a way to have optionally encoded fields. When a field condition is false, it will not be encoded or decoded.

Example:
```
type test = container {
    my_flags: u32,
    alternative_u32: u32 {my_flags == 1},
    alternative_u64: u64 {my_flags == 2},
}
```

### Transformation
Transformations are a powerful way to represent streaming bidirectional data transformation. They may take a number of arguments, and may be conditionally applied. They are declared exclusively through FFI.

They are evaluated left-to-right for serialization of the type, and evaluated right-to-left for deserialization of the type.

Example usage:
```
import_ffi test_xform as transform;

type example = container {
    ex1: u32 -> transform(7 /* argument */),
    ex2: u32 -> transform {ex1 == 5 /* conditional transform */},
    ex3: u32 -> transform(7) {ex2 == 3} -> transform(5),
} -> transform;
```

Example implementation:
* See `./src/prelude/gzip.rs`

## Supported Backends
* Rust
  * Include `protospec_build` as a build-dependency and call `protospec_build::compile_spec` in your `build.rs`:
  ```
    fn main() {
        protospec_build::compile_spec("example_spec", include_str!("./spec/example_spec.pspec"), &protospec_build::Options {
            ..Default::default()
        }).expect("failed to build example_spec.pspec");
    }
  ```
  Then include the module in your project with `protospec::include_spec` or
  `include!(concat!(env!("OUT_DIR"), "/example_spec.rs"))`.


## Features in planning
* generics for types
* support array-interior transformations & conditions in ASG
* add ability to reference original field in transform
* default entries in enums
* more complex handling of auto fields
  * offsets
  * multiple length candidates (i.e. a ternary)
  * generalized inferrable fields
    * bitfields
    * booleans
* flag to encode optional fields as zeros
* DCG of top-level field dependencies & field reordering
* clean up encoding/decoding
* a ton of docs