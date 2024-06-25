- dynamic type system, best-effort type inference and checking; goal is to provide useful suggestions; theoretical soundness non-goal
- basic bottom-up typechecker
- loose assignability: we check whether it is _possible_ that the types can work out correctly, not
  that they work out correctly in all cases
  - eg consider function `f: int64 -> int64`; we do not report an error on a call of `f` with
    argument type `int64 | float64`; it is possible that in this particular branch the caller
    ensures that the argument type is always `int64` but the typechecker is not sufficiently smart
    to reason it out
- function calls
- typecheck output for lsp
  - need to be able to figure out type of arbitrary expression
    - just store map of ast::Expr.syntax().text_range() -> Ty
  - need to be able to show doc for function
    - easy
  - need to be able to show doc for field/method/func
    - fields/methods kinda complicated, isnt covered by the ast::Expr map since the field might be in the middle of one FieldChain... need to figure out later
  - need to be able to suggest
    - variables (easy)
    - functions
    - fields/methods on base expr
    - function option keys AND nested in sdict (latter might be yucky). need to think more about this
    - assoc template names
- scoping and dataflow analysis
  - stack of blocks
  - each block stores: initial context ty, table var_assignments, resolved_var_types, declared_vars
  - merging possibly forked blocks into parent block
    - throw the child block declarations away, useless, same with the resolved var types, which arent relevant
    - only thing relevant are the child var assignments
    - see algo below for if stmts
- reference types
  - out of scope
  - hard to do since it's basically aliasing analysis on top of dataflow analysis (which is already pretty annoying for variables)
- associated templates
  - store two-layer table tmplname -> context_ty -> output_ty
    - once see template call, check whther already cached
      - if so, return ty
    - otherwise, monomorphization-like procedure: recheck the template from scratch with the new context ty,
      and record in table
      - ignore any type errors; that can wait until the last round
    - prevent infinite check loops: record call stack and bail out (report type any) if recursion
    - if too many distinct context types (eg 5), bail; recheck with context type `any` and use that for all future invocations
    - at the end, if we didnt hit the limit as above, recheck with context type `observed_context_ty1 | observed_context_ty2 | ...`, and only enable errors for this round

block merging algo

```
given
  parent_block
  if_block
  vec<else_block>

# alternate name: if_conditional_block_assigns (it contains all assignments within the conditional)
init all_inner_var_assigns = f_block.var_assigns
if !if_block.has_unconditional_else
  set occurs_in_all_paths to false for all vars
for else_block in else_blocks
  for var in else_block.var_assigns
    if var was already present
      all_inner_var_assigns[var].occurs_in_all_paths &= var.occurs_in_all_paths
    else
      insert with occurs_in_all_paths=false
    all_inner_var_assigns[var].ty |= var.ty

# now propagate to parent block
for var in inner_var_assigns
  commit var to parent_block.resolved_var_types as follows
    if var.occurs_in_all_paths:
      overwrite type
    else:
      union orig var type with assigned var type
  then propagate to parent assignments
    if var is declared in parent block
      stop (do not propagate further)
    else
      if var.occurs_in_all_paths
        overwrite parent assignment completely
      else
        union parent assignment type with new type
```
