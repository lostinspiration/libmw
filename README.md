Library for constructing a pipeline of middleware functions that have the option to call the next
in the chain if they so desire before returning back to the previous function call.

This allows for the programmer to perform work on the given context both before __and__ after invoking the
next function in the chain.

# Example
```rust
use libmw::prelude::*;

/// Sample freestanding handler function
fn standalone_func(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), PipelineError> {
  // do work before calling the next handler in the pipe
  println!("before in handler func");
  next.invoke(ctx)?;

  // do work after the next handler in the pipe returns
  println!("after in handler func");
  
  Ok(())
}

struct Context {}
impl PipelineContext for Context {
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}

fn main() {
  let mut builder = PipelineBuilder::new();

  // add freestanding function
  builder.with(standalone_func);

  // branch not taken because the predicate does not return true
  // if changed to true, nothing added after this branch will run
  builder.when(
    |_ctx| {
      false
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

        Ok(())
      });
    },
  );

  // with closure
  builder.with(|ctx, next| {
    // do work before calling the next handler in the pipe
  	println!("before in closure");
  	next.invoke(ctx)?;

    // do work after the next handler in the pipe returns
  	println!("after in closure");

  	Ok(())
  });
  
  // assemble the pipeline
  let pipeline = builder.assemble();

  // create context and call pipeline
  let mut context = Context { };
  let result = pipeline.invoke(&mut context);
  // handle result...
}
```