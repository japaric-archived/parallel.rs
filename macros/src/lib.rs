#![deny(warnings)]
#![feature(macro_rules, plugin_registrar, slicing_syntax)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::registry::Registry;
use syntax::ast::{CompilerGenerated, TtToken, TokenTree, UnsafeBlock};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacExpr, MacResult, NormalTT};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::{mod, Comma};

/// Executes several closures in parallel
///
/// - This macro is an expression, and will return a tuple containing the values returned by the
/// closures.
/// - One task per closure (i.e. there is no "load balancing").
/// - This macro will block until all the spawned tasks have finished. For this reason the closures
/// don't need to fulfill the `Send` trait (i.e. the closures may capture slices and references).
///
/// # Expansion
///
/// Consider the following call:
///
/// ``` ignore
/// let (a, b, c) = execute!{
///     |:| job_1(),
///     |:| job_2(),
///     |:| job_3(),
/// };
/// ```
///
/// This macro uses the "unsafe" [`Task`](../parallel/struct.Task.html) defined in the
/// [`parallel`](../parallel/index.html) crate, and its expansion looks (roughly) like this:
///
/// ``` ignore
/// let (a, b, c) = {
///     let __task_0 = |:| job_1();
///     let __task_1 = unsafe { Task::fork(|:| job_2()) };  // spawns a task
///     let __task_2 = unsafe { Task::fork(|:| job_3()) };  // spawns another task
///
///     // the current task takes care of the first closure
///     (__task_0(), __task_1.join(), __task_2.join())
///     // ^~ then blocks until the other two tasks finish
/// };
/// ```
///
/// # Failure
///
/// This macro will fail if any of the spawned tasks fails.
///
/// # Example
///
/// I'll borrow the binary tree example from Niko's
/// [blog](http://smallcultfollowing.com/babysteps/blog/2013/06/11/data-parallelism-in-rust/).
///
/// ```
/// #![feature(overloaded_calls, phase)]
///
/// extern crate parallel;
/// #[phase(plugin)]
/// extern crate parallel_macros;
///
/// struct Tree {
///     left: Option<Box<Tree>>,
///     right: Option<Box<Tree>>,
///     value: uint,
/// }
///
/// impl Tree {
///     fn sum(&self) -> uint {
///         fn sum(subtree: &Option<Box<Tree>>) -> uint {
///             match *subtree {
///                 None => 0,
///                 Some(box ref tree) => tree.sum(),
///             }
///         }
///
///         let (left_sum, right_sum) = execute!{
///             // NB Each closure captures a reference which doesn't fulfills `Send`
///             |:| sum(&self.left),
///             |:| sum(&self.right),
///         };
///
///         left_sum + self.value + right_sum
///     }
/// }
///
/// fn main() {
///     let tree = Tree {
///         value: 5,
///         left: Some(box Tree {
///             value: 3,
///             left: Some(box Tree {
///                 value: 1,
///                 left: None,
///                 right: Some(box Tree {
///                     value: 4,
///                     left: None,
///                     right: None,
///                 }),
///             }),
///             right: None,
///         }),
///         right: Some(box Tree {
///             value: 7,
///             left: None,
///             right: None,
///         }),
///     };
///
///     assert_eq!(tree.sum(), 20);
/// }
/// ```
#[macro_export]
macro_rules! execute {
    ($($closure:expr),+,) => ({ /* syntax extension */ });
}

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(r: &mut Registry) {
    r.register_syntax_extension(token::intern("execute"), NormalTT(box expand_execute, None));
}

fn expand_execute<'cx>(
    cx: &'cx mut ExtCtxt,
    sp: Span,
    tts: &[TokenTree],
) -> Box<MacResult + 'cx> {
    let parallel_task_fork_fn_path = {
        let segments = vec![
            token::str_to_ident("parallel"),
            token::str_to_ident("Task"),
            token::str_to_ident("fork"),
        ];

        cx.expr_path(cx.path_global(sp, segments))
    };

    let mut stmts = vec![];
    let tasks = tts.split(|tt| match *tt {
        TtToken(_, Comma) => true,
        _ => false,
    }).filter(|tts| {
        !tts.is_empty()
    }).enumerate().map(|(i, tts)|  {
        let closure = cx.new_parser_from_tts(tts).parse_expr();
        let ident = token::str_to_ident(format!("__task_{}", i)[]);

        let expr = if i == 0 {
            closure
        } else {
            let fn_name = parallel_task_fork_fn_path.clone();
            let args = vec![closure];

            // XXX There has to be a simpler way to wrap an expression in `unsafe`
            let block = cx.block_expr(cx.expr_call(sp, fn_name, args)).map(|mut b| {
                b.rules = UnsafeBlock(CompilerGenerated);
                b
            });
            cx.expr_block(block)
        };

        stmts.push(cx.stmt_let(sp, false, ident, expr));

        ident
    }).collect::<Vec<_>>();

    let mut is_first = true;
    let expr = cx.expr_tuple(sp, tasks.into_iter().map(|task| {
        let args = vec![];
        let task = cx.expr_ident(sp, task);

        if is_first {
            is_first = false;

            cx.expr_call(sp, task, args)
        } else {
            let method = token::str_to_ident("join");

            cx.expr_method_call(sp, task, method, args)
        }
    }).collect());

    MacExpr::new(cx.expr_block(cx.block(sp, stmts, Some(expr))))
}
