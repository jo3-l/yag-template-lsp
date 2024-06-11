#[cfg(test)]
mod codegen;
mod func;
mod generated;
mod ty;

pub use func::Func;
pub use generated::funcs::DEFINED_FUNCS;
