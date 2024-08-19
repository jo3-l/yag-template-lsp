## YAGPDB Template Environment Definitions

The `.ydef` files in this directory define the set of template functions available to YAGDPB templates, and are used by
the language server for hover documentation and error reporting. Since these files are [embedded into the language
server binary](https://github.com/jo3-l/yag-template-lsp/blob/main/crates/yag-template-envdefs/src/bundled_envdefs.rs)
at build time, any changes made will only be reflected after a fresh build.

## File format

The definition files are designed to be easily consumed by machines and humans alike.

To add a new function, first insert a line starting with `func` followed by the name of the function and its
signature—that is, the parameters it accepts. An optional parameter is denoted by a trailing `?` and a variadic
parameter by `...`.

Then, write documentation for the new function as indented lines below. Tabs are required; spaces will not work. So, to
give a complete example,

```txt
func removeRole(role, delay?)
	Removes the specified role from the triggering members.

	- `role`: A role ID, mention, name or role object.
	- `delay`: An optional delay in seconds.
```

defines a function `removeRole` accepting two parameters, `role` (required) and `delay` (optional), with the following
lines of documentation:

```txt
Removes the specified role from the triggering members.

- `role`: A role ID, mention, name or role object.
- `delay`: An optional delay in seconds.
```

Finally, as an organizational nicety, any lines starting with `==` will be ignored (acting effectively as comments). We
conventionally use such lines to name and visually separate groups of related functions, but they can contain any
content.
