use futures::{Async, Future, Poll};
use std::net::IpAddr;
use tower_service::Service;

pub struct Resolver<R>(R);

impl<R> Resolver<R> {
    pub fn new(resolver: R) -> Self {
        Resolver(resolver)
    }
}

impl<R: Resolve> Service<R::Target> for Resolver<R>
{
    type Response = IpAddr;
    type Error = R::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error> + Send + 'static>;

    fn call(&mut self, target: R::Target) -> Self::Future {
        let fut = self.0.lookup_ip(target);
        Box::new(fut)
    }

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }
}

pub trait Resolve {
    type Target;
    type Error: 'static;
    type Future: Future<Item = IpAddr, Error = Self::Error> + Send + 'static;

    fn lookup_ip(&mut self, target: Self::Target) -> Self::Future;
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

    impl<A: IntoName + TryParseIp> Resolve for TrustDns<A> {
        type Target = A;
        type Error = ResolveError;
        type Future = Box<Future<Item = IpAddr, Error = Self::Error> + Send + 'static>;

        fn lookup_ip(&mut self, target: Self::Target) -> Self::Future {
            let fut = self.0.lookup_ip(target)
                .and_then(|ip| Ok(ip.iter().next().unwrap()));
            
            Box::new(fut)
        }
    }
}