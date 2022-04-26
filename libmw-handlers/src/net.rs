//! Collection of network based middleware handlers

/// Allows the use of the `receive` middleware functions
///
/// Results from the network `recv` call will be stored in the backing field of the [Receivable::request] function
pub trait Receivable {
	/// Provides access to a buffer for the received bytes
	fn request(&mut self) -> &mut Vec<u8>;
}

/// Allows the use of the `send` middleware functions
///
/// The network `send` call will send all the data stored in the backing field of the [Sendable::response] function
pub trait Sendable {
	/// Provides access to a buffer for the sendable bytes
	fn response(&mut self) -> &mut Vec<u8>;
}

/// `TCP` based network middleware
pub mod tcp {
	use super::{Receivable, Sendable};
	use libmw::prelude::*;
	use std::{error::Error, net::TcpStream};

	/// Defines a [TcpStream] for the [send] and [receive] functions to work with
	pub trait Networkable {
		/// Provides access to the [TcpStream]
		fn socket(&mut self) -> &TcpStream;
	}

	/// Middleware that will send all the bytes from the implemented [Sendable::response] function then calling the next
	/// middleware in the [Pipeline]
	pub fn send<CtxType>(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), Box<dyn Error>>
	where
		CtxType: PipelineContext + Sendable + Networkable + 'static,
	{
		next.invoke(ctx)?;
		Ok(())
	}

	/// Middleware that will read from the network placing the results in the buffer from the implemented [Receivable::request] function then calling the next
	/// middleware in the [Pipeline]
	pub fn receive<CtxType>(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), Box<dyn Error>>
	where
		CtxType: PipelineContext + Receivable + Networkable + 'static,
	{
		next.invoke(ctx)?;
		Ok(())
	}
}

/// `UDP` based network middleware
#[allow(warnings)]
pub mod udp {
	use super::{Receivable, Sendable};
	use libmw::prelude::*;
	use std::{error::Error, net::UdpSocket};

	/// Defines a [UdpSocket] for the [send] and [receive] functions to work with
	pub trait Networkable {
		/// Provides access to the [UdpSocket]
		fn socket(&mut self) -> &UdpSocket;
	}

	/// Middleware that will send all the bytes from the implemented [Sendable::response] function then calling the next
	/// middleware in the [Pipeline]
	fn send<CtxType>(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), Box<dyn Error>>
	where
		CtxType: PipelineContext + Sendable + Networkable + 'static,
	{
		unimplemented!()
	}

	/// Middleware that will read from the network placing the results in the buffer from the implemented [Receivable::request] function then calling the next
	/// middleware in the [Pipeline]
	fn receive<CtxType>(ctx: &mut dyn PipelineContext, next: Pipeline) -> Result<(), Box<dyn Error>>
	where
		CtxType: PipelineContext + Receivable + Networkable + 'static,
	{
		unimplemented!()
	}
}
