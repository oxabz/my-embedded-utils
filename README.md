# My embedded utils 

A repository for my rust embedded utils crate 

## thisdefmterror

Basicaly ``thiserror`` but worse but it also implements ``defmt::Format``.

**Example**

```rust
#[derive(Debug, DefmtError)] // Note the debug derive is needed for core::error::Error
enum MyError{
    #[error("A unit variant error")]
    UnitError,
    #[error("Tuple error, {1}, {0}, {2}")]
    TupleError(u8, f32, [u8;4]),
    #[error("An into error : {}")]
    IntoError(#[into]SomeOtherError), // Needs to implment defmt::Format and core::error::Error
    #[error("An type that only implement Display can still be used : {}")]
    Display(#[display]NotADefmt) // Note this should be used sparsly because it force the compiler to keep the format strings for the type
}

```

## pack

>Note: This crate use big-endian as most networks do

Dead simple serialization and deserialization crate