# Rules module design

Treat `action.rs`, `block.rs`, and the `expr` module as separate, independently
reasoned-about components. An agent working on one of these modules should be
able to understand and change its logic without needing to understand the
implementation details of the other two.

- Keep dependencies between the three modules to a minimum.
- Keep implementation details, representation types, and helpers private to
  the module that owns them.
- When modules must interact, expose only small, focused `pub(crate)` methods
  with a clear purpose and narrow inputs and outputs. Do not expose broad
  internal state or require callers to coordinate multiple implementation
  details.
- Put logic in the module that owns the relevant behavior rather than adding
  cross-module knowledge or duplicating that logic elsewhere.

Before adding a new dependency or crate-public method, check whether the
interaction can be expressed through a smaller boundary or kept within the
owning module.
