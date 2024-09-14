#![allow(dead_code)]

#[maybe_async_cfg::maybe(send="Send", async(not(feature = "__test__is_sync"), async_trait::async_trait))]
trait Trait {
    fn sync_fn() {}

    async fn declare_async(&self);

    async fn async_fn(&self) {
        self.declare_async().await
    }
}

#[maybe_async_cfg::maybe(send="?Send", async(not(feature = "__test__is_sync"), async_trait::async_trait))]
trait NotSendTrait {
    async fn declare_async_not_send(&self);

    async fn async_fn_not_send(&self) {
        self.declare_async_not_send().await
    }
}
#[maybe_async_cfg::maybe(send="Send", async(not(feature = "__test__is_sync"), async_trait::async_trait))]
pub trait PubTrait {
    fn sync_fn() {}

    async fn declare_async(&self);

    async fn async_fn(&self) {
        self.declare_async().await
    }
}


#[maybe_async_cfg::maybe(keep_self, async(not(feature = "__test__is_sync")))]
async fn async_fn() {}


#[maybe_async_cfg::maybe(keep_self, async(not(feature = "__test__is_sync")))]
pub async fn pub_async_fn() {}

struct Struct;


#[maybe_async_cfg::maybe(keep_self, idents(Trait), async(not(feature = "__test__is_sync"), async_trait::async_trait))]
impl Trait for Struct {
    fn sync_fn() {}

    async fn declare_async(&self) {}

    async fn async_fn(&self) {
        async { self.declare_async().await }.await
    }
}

#[maybe_async_cfg::maybe(keep_self, idents(NotSendTrait), send="?Send", async(not(feature = "__test__is_sync"), async_trait::async_trait))]
impl NotSendTrait for Struct {
    async fn declare_async_not_send(&self) {}

    async fn async_fn_not_send(&self) {
        async { self.declare_async_not_send().await }.await
    }
}

#[cfg(feature = "__test__is_sync")]
fn main() {}


#[cfg(not(feature = "__test__is_sync"))]
#[async_std::main]
async fn main() {
    let s = Struct;
    s.declare_async().await;
    s.async_fn().await;
    async_fn().await;
    pub_async_fn().await;
}
