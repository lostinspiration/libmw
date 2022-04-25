Library for constructing a pipeline of middleware functions that have the option to call the next
in the chain if they so desire before returning back to the previous function call.

This allows for the programmer to perform work on the given context both before __and__ after invoking the
next function in the chain.

# Example
```rust
use libmw::prelude::*;

/// Sample free standing handler function
fn standalone_func(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), PipelineError> {
  // do work before calling the next handler in the pipe
  println!("before in handler func");
  next.invoke(ctx)?;

  // do work after the next handler in the pipe returns
  println!("after in handler func");
  
  Ok(())
}

/// Use free standing functions to add additional trait bounds your context implements
/// for generic handlers that can be shared without needing knowledge of the implemented `PipelineContext`
fn standalone_func_enhance<CtxType>(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), PipelineError> 
where:
  CtxType: PipelineContext + Repeatable + 'static {
    if let Some(context) = ctx.as_any_mut().downcast_mut::<CtxType>() {
      while context.repeat() {
      next.invoke(context)?;

      if context.delay() != 0 {
        std::thread::sleep(std::time::Duration::from_millis(context.delay() as u64));
      }
    }
  }
  
  Ok(())
}

#[derive(PipelineContext)]
struct Context {
  take_branch: bool
}

fn main() {
  let mut builder = PipelineBuilder::new();

  // add freestanding function
  builder.with(standalone_func);

  // branch taken depending on how context was created
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
  let mut context = Context { take_branch: true };
  let result = pipeline.invoke(&mut context);
  match result {
    Ok(_) -> {
      // deal with your mutated context
    }
    Err(_) => {
      // handle your pipeline errors
    }
  }
}
```