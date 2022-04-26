//! Provides a standard set of middleware handlers and traits for [libmw] crate
#![deny(missing_docs, unsafe_code)]

pub mod net;

use libmw::prelude::*;
use std::error::Error;

/// Allows for a [PipelineContext] to have a repeatable control flow through the pipeline
pub trait Repeatable {
	/// Specifies if the [PipelineContext] should be sent back down the [Pipeline]
	fn should_repeat(&self) -> bool;

	/// Specifies an optional delay to wait before sending the [PipelineContext] back down the [Pipeline] in milliseconds
	///
	/// Defaults to `0`
	fn delay(&self) -> Option<u64> {
		None
	}
}

/// Middleware that will keep sending the [PipelineContext] down the [Pipeline] until the implemented [Repeatable::should_repeat] function returns false
/// waiting for an optional [Repeatable::delay] in milliseconds
pub fn repeat<CtxType>(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), Box<dyn Error>>
where
	CtxType: PipelineContext + Repeatable + 'static,
{
	if let Some(context) = ctx.as_any_mut().downcast_mut::<CtxType>() {
		while context.should_repeat() {
			next.invoke(context)?;

			if let Some(delay) = context.delay() {
				std::thread::sleep(std::time::Duration::from_millis(delay));
			}
		}
	}

	Ok(())
}

/// Middleware that results in a `noop` by immediately returning `Ok(())`
/// 
/// Useful for returning from a branch early or having an explicit definition of the end
/// of the pipeline or branch
pub fn end(_: &mut dyn PipelineContext, _: Pipeline) -> Result<(), Box<dyn Error>> {
	Ok(())
}
