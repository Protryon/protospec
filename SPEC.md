# ProtoSpec Language Specification

## Concepts

### Data Types
* Primitives
* Inline definitions
* Out-of-line definitions
* Foreign definitions

### Data Transformations
* Core
* User defined
* Foreign

### Data Dependencies
* Length
  * Constant
  * Expression on field
  * Until terminator
  * Until end of container, if defined
* Presence

### Integration
* Encoding and decoding of streams of data
* Can accept blobs (as a stream)

## Data Types

* u8-u128
* i8-i128
* arrays
* container (length-defined struct)
* value-mapped enums
* blob (raw binary data input types)
* bool

## Syntax
```

ident := [a-zA-Z_][a-zA-Z0-9_-]*

string := '"' [[^"]*?] '"' # allow escapes

program := declaration*

declaration := (type_declaration
    | import_declaration
    | ffi_declaration
    | const_declaration) ";"

typed_argument := ident ":" base_type ("?" expression)?

type_declarator := ident
    | ident "(" (typed_argument ("," typed_argument)* ","?)? ")"

type_declaration := "type" type_declarator "=" type

import_item = ident ("as" ident)?

import_declaration := "import" import_item ("," import_item)* ","? "from" string

ffi_declaration := "import_ffi" ident "as" ("transform" | "type")

const_declaration := "const" ident ":" base_type "=" expression

length_constraint := expression
    | ".." expression?

scalar_type := "[u, i][8, 16, 32, 64, 128]" | "v[32, 64, 128]"

type_call := ident |
    ident "(" (expression ("," expression)* ","?)? ")"

base_type := type_container |
    type_enum |
    scalar_type |
    array_type |
    "f32" | "f64" | "bool" | type_call

conditional_clause := "{" expression "}"

transform_call := ident
    | ident "(" (expression ("," expression)* ","?)? ")"

flag := "+" ident

flags := flag+

type := base_type flags? conditional_clause? ("->" transform_call conditional_clause?)*
    | "(" type ")"

tagged_type := ident ":" type

type_container := "container" ("[" expression "]")? "{" (tagged_type ("," tagged_type)*)? ","? "}"

array_type := type "[" length_constraint "]"

tagged_expression := ident ("=" expression)?

type_enum := "enum" scalar_type "{" (ident "=" expression ("," tagged_expression)*)? ","? "}"

integer := [0-9]+ scalar_type?

expression := or_expression ("?" expression ":" or_expression)

or_expression := and_expression ("||" and_expression)*

and_expression := bit_or_expression ("&&" bit_or_expression)*

bit_or_expression := bit_xor_expression ("|" bit_xor_expression)*

bit_xor_expression := bit_and_expression ("^" bit_and_expression)*

bit_and_expression := eq_expression ("&" eq_expression)*

eq_expression := rel_expression (("==" | "!=") rel_expression)*

rel_expression := shift_expression (("<" | ">" | ">=" | "<=") shift_expression)*

shift_expression := add_expression ((">>" | "<<" | ">>>") add_expression)*

add_expression := multiply_expression (("+" | "-") multiply_expression)*

multiply_expression := cast_expression (("*" | "/" | "%") cast_expression)*

cast_expression := unary_expression (":>" base_type)*

unary_expression := ("-" | "!")* array_index_expression

array_index_expression := primary_expression ("[" expression "]")*

primary_expression := integer | ident | string | "(" expression ")"

```
