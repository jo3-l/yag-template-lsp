use std::collections::hash_map::Entry;
use std::{iter, mem};

use ecow::EcoString;
use yag_template_syntax::ast::{self, AstNode, SyntaxNodeExt};
use yag_template_syntax::SyntaxKind;

use crate::typeck::check::check_template_body;
use crate::typeck::context::{AssocTemplate, TypeckContext, TypeckOptions};
use crate::typeck::flow::{Block, BlockKind};
use crate::typeck::output::AssocTemplateInfo;
use crate::typeck::ty::{union_all, Ty};

pub(crate) fn hoist_assoc_template_defs(ctx: &mut TypeckContext, root: ast::Root) {
    let assoc_templates = root.syntax().descendants().filter_map(|node| match node.kind() {
        SyntaxKind::TemplateBlock => {
            let template_block = node.to::<ast::TemplateBlock>();
            let name = EcoString::from(template_block.block_clause()?.template_name()?.get());
            Some(AssocTemplate::new(
                name,
                template_block.syntax().text_range(),
                template_block.action_list()?,
            ))
        }
        SyntaxKind::TemplateDefinition => {
            let template_def = node.to::<ast::TemplateDefinition>();
            let name = EcoString::from(template_def.define_clause()?.template_name()?.get());
            Some(AssocTemplate::new(
                name,
                template_def.syntax().text_range(),
                template_def.action_list()?,
            ))
        }
        _ => None,
    });

    for assoc_template in assoc_templates {
        match ctx.assoc_templates.entry(assoc_template.name.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(assoc_template);
            }
            Entry::Occupied(_) => {
                // TODO: issue error re: duplicate template definition
            }
        }
    }
}

pub(crate) fn check_assoc_templates(ctx: &mut TypeckContext) -> Vec<AssocTemplateInfo> {
    let (mut assoc_templates, assoc_template_bodies): (Vec<_>, Vec<_>) = ctx
        .assoc_templates
        .values()
        .map(|assoc_template| {
            let context_ty = if assoc_template.overflowed_instantiation_cache {
                Ty::Any
            } else {
                union_all(assoc_template.cached_instantiations.values())
            };

            let info = AssocTemplateInfo {
                name: assoc_template.name.clone(),
                defn_range: assoc_template.defn_range,
                context_ty,
                return_ty: Ty::Never, // determined in next pass
            };
            let body = assoc_template.body.clone();
            (info, body)
        })
        .unzip();

    for (assoc_template, body) in iter::zip(assoc_templates.iter_mut(), assoc_template_bodies.iter()) {
        assoc_template.return_ty = check_in_child_context(
            ctx,
            assoc_template.name.clone(),
            assoc_template.context_ty.clone(),
            TypeckOptions {
                record_output: true,
                evaluate_new_instantiations: false,
            },
            |child_ctx| {
                check_template_body(child_ctx, body.actions());
                child_ctx.top_block.return_ty.take()
            },
        )
    }
    assoc_templates
}

pub(crate) fn instantiate_template(ctx: &mut TypeckContext, assoc_template: &mut AssocTemplate, context_ty: Ty) -> Ty {
    if let Some(cached) = assoc_template.cached_instantiations.get(&context_ty) {
        return cached.clone();
    }

    let overflow = assoc_template.overflowed_instantiation_cache
        || assoc_template.cached_instantiations.len() >= AssocTemplate::MAX_UNIQUE_INSTANTIATIONS;
    if overflow {
        assoc_template.overflowed_instantiation_cache = true;
        return Ty::Any;
    }

    if !ctx.opts.evaluate_new_instantiations || ctx.call_stack.contains(&assoc_template.name) {
        return Ty::Any;
    }

    let return_ty = check_in_child_context(
        ctx,
        assoc_template.name.clone(),
        context_ty.clone(),
        TypeckOptions {
            record_output: false,
            evaluate_new_instantiations: true,
        },
        |child_ctx| {
            check_template_body(child_ctx, assoc_template.body.actions());
            child_ctx.top_block.return_ty.take()
        },
    );
    assoc_template
        .cached_instantiations
        .insert(context_ty, return_ty.clone());
    return_ty
}

fn check_in_child_context<'e, F, R>(
    parent_ctx: &'e mut TypeckContext,
    template_name: EcoString,
    context_ty: Ty,
    opts: TypeckOptions,
    check: F,
) -> R
where
    F: FnOnce(&mut TypeckContext) -> R,
{
    // Temporarily transfer the call stack, associated template instantiation cache, and output sink
    // of the parent context to the child context so that it may write to them if needed.
    let mut call_stack = mem::take(&mut parent_ctx.call_stack);
    call_stack.push(parent_ctx.template_name.clone());

    let assoc_templates = mem::take(&mut parent_ctx.assoc_templates);
    let out = mem::take(&mut parent_ctx.out);

    let mut child_ctx = TypeckContext {
        opts,
        env: parent_ctx.env,
        template_name,
        call_stack,
        top_block: Block::new_detached(BlockKind::default(), context_ty),
        parent_blocks: Vec::new(),
        assoc_templates,
        out,
    };
    let ret = check(&mut child_ctx);

    // Now restore the state of the parent context.
    let mut call_stack = mem::take(&mut child_ctx.call_stack);
    call_stack.pop();
    parent_ctx.call_stack = call_stack;

    parent_ctx.assoc_templates = mem::take(&mut child_ctx.assoc_templates);
    parent_ctx.out = mem::take(&mut child_ctx.out);

    ret
}
