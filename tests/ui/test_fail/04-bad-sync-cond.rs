#[maybe_async_cfg::maybe(
    sync(feature = "__test__is_sync"),
    async(not(feature = "__test__is_sync")),
)]
async fn async_fn() -> bool {
    true
}

// bad sync condition
//#[maybe_async::test(unknown(feature="async", async_std::test))]
#[maybe_async_cfg::maybe(
    idents(async_fn(fn)),
    sync(unknown = "__test__is_sync"),
    async(not(feature = "__test__is_sync")),
)]
async fn test_async_fn() {
    let res = async_fn().await;
    assert_eq!(res, true);
}

fn main() {

}
