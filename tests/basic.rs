use std::net::{ToSocketAddrs, IpAddr};
use tower_dns::{Resolver, Resolve};
use tower_service::Service;
use futures::future;

#[test]
fn basic() {
    let mut resolver = Resolver::new(BlockingResolver);
    resolver.call("[::]:0");
}

struct BlockingResolver;
impl<A: ToSocketAddrs> Resolve<A> for BlockingResolver {
    type Error = ();
    type Future = future::FutureResult<IpAddr, ()>;

    fn lookup(&mut self, target: A) -> Self::Future {
        let mut addrs = target.to_socket_addrs().unwrap();
        match addrs.next() {
            Some(addr) => future::ok(addr.ip()),
            None => panic!("Could not resolve!"),
        }
    }
}