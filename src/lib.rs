//! Tower middleware to resolve DNS requests and produce and `IpAddr`

use futures::{Async, Future, Poll};
use std::net::IpAddr;
use tower_service::Service;

/// A `Service` that resolves `IpAddr`
/// based on some type that implements `Resolve`.
pub struct Resolver<R>(R);

impl<R> Resolver<R> {
    pub fn new(resolver: R) -> Self {
        Resolver(resolver)
    }
}

impl<A, R: Resolve<A>> Service<A> for Resolver<R>
{
    type Response = IpAddr;
    type Error = R::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error> + Send + 'static>;

    fn call(&mut self, target: A) -> Self::Future {
        Box::new(self.0.lookup(target))
    }

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }
}

/// Represents a type that can resolve an `IpAddr` from some
/// type `Target`.
pub trait Resolve<Target> {
    type Error: 'static;
    type Future: Future<Item = IpAddr, Error = Self::Error> + Send + 'static;

    fn lookup(&mut self, target: Target) -> Self::Future;
}

#[cfg(feature = "trust-dns")]
pub use crate::trust_dns::TrustDns;

#[cfg(feature = "trust-dns")]
mod trust_dns {
    use trust_dns_resolver::error::ResolveError;
    use trust_dns_resolver::{AsyncResolver, IntoName, TryParseIp};
    use std::net::IpAddr;
    use futures::Future;
    use std::marker::PhantomData;
    use super::Resolve;

    pub struct TrustDns<A>(AsyncResolver, PhantomData<A>);
    impl<A> TrustDns<A> {
        pub fn new(resolver: AsyncResolver) -> Self {
            TrustDns(resolver, PhantomData)
        }
    }

    impl<A: IntoName + TryParseIp> Resolve<A> for TrustDns<A> {
        type Error = ResolveError;
        type Future = Box<Future<Item = IpAddr, Error = Self::Error> + Send + 'static>;

        fn lookup(&mut self, target: Self::Target) -> Self::Future {
            let fut = self.0.lookup_ip(target)
                .and_then(|ip| Ok(ip.iter().next().unwrap()));
            
            Box::new(fut)
        }
    }
}