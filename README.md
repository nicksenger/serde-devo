## serde-devo

`serde-devo` is a proc-macro helper utility intended to help with compatibility of evolving types shared across multiple applications, particularly in situations where those apps' deployments cannot be coordinated (e.g. microservices).

The macro generates a new set of `Devolved*` types, for which any associated enums will contain a `serde(untagged)` variant. While the original types can continue to be used in applications which "own" the data, the `Devolved*` types are intended for use in applications which may or may not have the latest version of the original types.

### Limitations

This only works for self-describing formats like JSON or MessagePack.
