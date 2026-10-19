use std::future::Future;

pub trait ShutsDownUnlessShared {
    fn shut_down_unless_shared(self) -> impl Future<Output = ()> + Send;
}
