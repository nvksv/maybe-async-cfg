#![allow(unused_imports)]
use std::future::Future;

#[maybe_async_cfg::maybe(keep_self, sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
pub async fn with_fn<T, F: Sync + std::future::Future<Output = Result<(), ()>>>(
    test: T,
) -> Result<(), ()>
    where
        T: FnOnce() -> F,
{
    test().await
}

#[maybe_async_cfg::maybe(keep_self, sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
pub async fn with_fn_where<T, F>(test: T) -> Result<(), ()>
    where
        T: FnOnce() -> F,
        F: Sync + Future<Output = Result<(), ()>>,
{
    test().await
}

#[maybe_async_cfg::maybe(keep_self, sync(feature = "__test__is_sync"))]
fn main() {
    with_fn(|| Ok(())).unwrap();
    with_fn_where(|| Ok(())).unwrap();
}

#[maybe_async_cfg::maybe(keep_self, async(not(feature = "__test__is_sync")))]
#[tokio::main]
async fn main() {
    with_fn(|| async { Ok(()) }).await.unwrap();
    with_fn_where(|| async { Ok(()) }).await.unwrap();
}
