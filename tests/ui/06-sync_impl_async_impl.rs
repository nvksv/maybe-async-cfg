#![allow(dead_code, unused_variables)]

/// InnerClient differ a lot between sync and async.
#[maybe_async_cfg::maybe(sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync"), async_trait::async_trait))]
trait Trait {
    async fn maybe_async_fn();
}

/// The higher level API for end user.
pub struct Struct;

/// Synchronous  implementation, only compiles when `__test__is_sync` feature is off.
/// Else the compiler will complain that *request is defined multiple times* and
/// blabla.
#[maybe_async_cfg::maybe(keep_self, idents(Trait), sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
#[maybe_async::only_if(key="sync")]
impl Trait for Struct {
    fn maybe_async_fn() { }
}

/// Asynchronous implementation, only compiles when `__test__is_sync` feature is off.
#[maybe_async_cfg::maybe(keep_self, idents(Trait), sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync"), async_trait::async_trait))]
#[maybe_async::only_if(key="async")]
impl Trait for Struct {
    async fn maybe_async_fn() { }
}

#[maybe_async_cfg::maybe(keep_self, idents(Trait, another_maybe_async_fn(fn)), sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
impl Struct {
    async fn another_maybe_async_fn()  {
        <Self as Trait>::maybe_async_fn().await
        // When `__test__is_sync` is toggle on, this block will compiles to:
        // Self::maybe_async_fn()
    }
}

#[maybe_async_cfg::maybe(keep_self, idents(another_maybe_async_fn(fn)), sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
#[maybe_async::only_if(key="sync")]
fn main() {
    let _ = Struct::another_maybe_async_fn();
}

#[maybe_async_cfg::maybe(keep_self, idents(another_maybe_async_fn(fn)), sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
#[maybe_async::only_if(key="async")]
#[tokio::main]
async fn main() {
    let _ = Struct::another_maybe_async_fn().await;
}
