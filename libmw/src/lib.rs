//! Library for constructing a pipeline of middleware functions that have the option to call the next
//! in the chain if they so desire before returning back to the previous function call.
//!
//! This allows for the programmer to perform work on the given context both before __and__ after invoking the
//! next function in the chain.
#![deny(missing_docs, unsafe_code)]

use std::{any::Any, error::Error, sync::Arc};
use thiserror::Error;

/// Library prelude to bring in the most used structures and traits
pub mod prelude {
	pub use crate::{Pipeline, PipelineBuilder, PipelineContext, PipelineError};
	pub use libmw_macro::PipelineContext;
}

// these are used in the public api to accept both free standing functions and closures
type PredicateThunk = fn(&mut dyn PipelineContext) -> bool;
type BranchThunk = fn(&mut PipelineBuilder);
type MiddlewareThunk = fn(&mut dyn PipelineContext, Pipeline) -> Result<(), Box<dyn Error>>;

// used internally as wrapper closure thunks
type MiddlewareTraitThunk = Box<dyn Fn(&mut dyn PipelineContext, Pipeline) -> Result<(), Box<dyn Error>>>;
type Thunk = Arc<dyn Fn(&mut dyn PipelineContext) -> Result<(), Box<dyn Error>>>;

/// Holder struct that contains a next method in the pipeline
pub struct Pipeline {
	next: Option<Thunk>,
}

impl Pipeline {
	/// Invoke the assigned next middleware method if exists, otherwise essentially a noop
	#[must_use]
	pub fn invoke(&self, ctx: &mut dyn PipelineContext) -> Result<(), Box<dyn Error>> {
		if self.next.is_none() {
			return Ok(());
		}

		(*(self.next.as_ref().unwrap()))(ctx)?;

		Ok(())
	}
}

/// Error
#[derive(Error, Debug)]
pub enum PipelineError {
	/// Generic pipeline error
	#[error("{0}")]
	Generic(String),
}

/// Context type that is passed to each registered middleware function
pub trait PipelineContext {
	/// Allows for downcasting to an immutable reference of the known concrete type
	fn as_any(&self) -> &dyn Any;

	/// Allows for downcasting to an mutable reference of the known concrete type
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Pipeline builder structure that holds the list of added middleware as well as providing
/// ways of interacting with that list
pub struct PipelineBuilder {
	middleware: Vec<MiddlewareTraitThunk>,
}

impl PipelineBuilder {
	/// Creates a new [PipelineBuilder] object
	pub fn new() -> Self {
		Self { middleware: Vec::new() }
	}

	/// Adds a new [MiddlewareThunk] to the pipeline. This can be a free standing fn or an inline closure
	pub fn with(&mut self, middleware: MiddlewareThunk) {
		self.middleware.push(Box::new(middleware));
	}

	/// Branches the [Pipeline] based on the result of the given [PredicateThunk] with the new
	/// set of [PipelineBuilder] instructions
	pub fn when(&mut self, predicate: PredicateThunk, builder: BranchThunk) {
		let mut branch_builder = PipelineBuilder::new();
		builder(&mut branch_builder);
		let branch = branch_builder.assemble();
		self.middleware.push(Box::new(move |ctx, next| {
			if predicate(ctx) {
				return branch.invoke(ctx);
			}

			next.invoke(ctx)
		}));
	}

	/// Assembles the pipeline giving a single entrypoint to pass a [PipelineContext] in.
	///
	/// The resulting [Pipeline] object can be cached and run multiple times with different contexts
	pub fn assemble(self) -> Pipeline {
		let Self { middleware } = self;

		let mut chain: Option<Pipeline> = None;
		let mut iter = middleware.into_iter().rev();

		while let Some(mw) = iter.next() {
			if chain.is_none() {
				chain = Some(Pipeline {
					next: Some(Arc::new(move |ctx| mw(ctx, Pipeline { next: None }))),
				});
				continue;
			}

			let n = chain.take().unwrap().next.take().unwrap();
			chain = Some(Pipeline {
				next: Some(Arc::new(move |ctx| mw(ctx, Pipeline { next: Some(n.clone()) }))),
			});
		}

		chain.unwrap()
	}
}

#[cfg(test)]
mod tests {
	use libmw_macro::PipelineContext;
	use super::*;

	#[derive(PipelineContext)]
	struct Context {
		take_branch: bool,
	}

	#[test]
	fn it_works() {
		let mut builder = PipelineBuilder::new();

		builder.with(|ctx, next| {
			println!("before in closure");
			next.invoke(ctx)?;
			println!("after in closure");

			Ok(())
		});

		builder.when(
			|ctx| match ctx.as_any().downcast_ref::<Context>() {
				Some(c) => c.take_branch,
				None => false,
			},
			|builder| {
				builder.with(|ctx, next| {
					println!("branch handler 1 before");
					next.invoke(ctx)?;
					println!("branch handler 1 after");

					Ok(())
				});

				builder.with(|ctx, next| {
					println!("branch handler 2 before");
					next.invoke(ctx)?;
					println!("branch handler 2 after");

					// Err(PipelineError::Generic(String::from("Barfed")).into())
					Ok(())
				});
			},
		);

		builder.with(|ctx, next| {
			println!("before in last closure");
			next.invoke(ctx)?;
			println!("after in last closure");

			Ok(())
		});

		let pipeline = builder.assemble();
		let mut context = Context { take_branch: true };
		let result = pipeline.invoke(&mut context);
		match result {
			Ok(_) => {
				// handle things in context
			}
			Err(e) => {
				println!("{:#?}", e);
			}
		}

		let mut context = Context { take_branch: false };
		let result = pipeline.invoke(&mut context);
		match result {
			Ok(_) => {
				// handle things in context
			}
			Err(e) => {
				println!("{:#?}", e);
			}
		}
		println!("It did not crash!");
	}
}
