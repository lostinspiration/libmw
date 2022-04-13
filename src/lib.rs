//! Library for constructing a pipeline of middleware functions that have the option to call the next
//! in the chain if they so desire before returning back to the previous function call.
//!
//! This allows for the programmer to perform work on the given context both before __and__ after invoking the
//! next function in the chain.
//!
//! # Example
//! ```no_run
//! /// Sample freestanding handler function
//! fn handler_in_func(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), PipelineError> {
//!   // do work before calling the next handler in the pipe
//!   println!("before in handler func");
//!   next.invoke(ctx)?;
//!
//!   // do work after the next handler in the pipe returns
//!   println!("after in handler func");
//!   
//!   Ok(())
//! }
//!
//! struct Context {}
//! impl PipelineContext for Context {
//! 	fn as_any(&self) -> &dyn Any {
//! 		self
//! 	}
//!
//! 	fn as_any_mut(&mut self) -> &mut dyn Any {
//! 		self
//! 	}
//! }
//!
//! fn main() {
//!   let mut builder = PipelineBuilder::new();
//!
//!   // add freestanding function
//!   builder.with(handler_in_func);
//!
//!   // with closure
//!   builder.with(|ctx, next| {
//!     // do work before calling the next handler in the pipe
//!   	println!("before in closure");
//!   	next.invoke(ctx)?;
//!
//!     // do work after the next handler in the pipe returns
//!   	println!("after in closure");
//!
//!   	Ok(())
//!   });
//!   
//!   // assemble the pipeline
//!	  let pipeline = builder.assemble();
//!
//!	  // create context and call pipeline
//!	  let mut context = Context { };
//!	  let result = pipeline.invoke(&mut context);
//!   // handle result...
//! }
//! ```
#![deny(missing_docs, unsafe_code)]

use std::{any::Any, sync::Arc};
use thiserror::Error;

/// Library prelude to bring in the most used structures and traits
pub mod prelude {
	pub use crate::{Pipeline, PipelineBuilder, PipelineContext, PipelineError};
}

type MiddlewareThunk = fn(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), PipelineError>;
type Thunk = Arc<dyn Fn(&mut dyn PipelineContext) -> Result<(), PipelineError>>;

/// Holder struct that contains a next method in the pipeline
pub struct Pipeline {
	next: Option<Thunk>,
}

impl Pipeline {
	/// Invoke the assigned next middleware method if exists, otherwise essentially a noop
	#[must_use]
	pub fn invoke(&self, ctx: &mut dyn PipelineContext) -> Result<(), PipelineError> {
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
	middleware: Vec<MiddlewareThunk>,
}

impl PipelineBuilder {
	/// Creates a new [PipelineBuilder] object
	pub fn new() -> Self {
		Self { middleware: Vec::new() }
	}

	/// Adds a new [MiddlewareThunk] to the pipeline. This can be a free standing fn or an inline closure
	pub fn with(&mut self, middleware: MiddlewareThunk) {
		self.middleware.push(middleware);
	}

	/// Assembles the pipeline giving a single entrypoint to pass a [PipelineContext] in.
	/// The resulting pipeline object can be cached and run multiple times with different contexts.
	pub fn assemble(self) -> Pipeline {
		let Self { middleware } = self;

		let mut chain: Option<Pipeline> = None;
		let mut iter = middleware.into_iter().rev();

		while let Some(mw) = iter.next() {
			if chain.is_none() {
				chain = Some(Pipeline {
					next: Some(Arc::new(move |ctx| {
						mw(ctx, Pipeline { next: None })?;
						Ok(())
					})),
				});
				continue;
			}

			let n = chain.take().unwrap().next.take().unwrap();
			chain = Some(Pipeline {
				next: Some(Arc::new(move |ctx| {
					mw(ctx, Pipeline { next: Some(n.clone()) })?;
					Ok(())
				})),
			});
		}

		chain.unwrap()
	}
}
