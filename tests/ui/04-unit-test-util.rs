#[maybe_async_cfg::maybe(sync(feature = "__test__is_sync"), async(not(feature = "__test__is_sync")))]
async fn async_fn() -> bool {
    true
}

#[maybe_async_cfg::maybe(
    idents(async_fn),
    sync(feature = "__test__is_sync", test),
    async(all(not(feature="__test__is_sync"), feature = "__test__async_std"), async_std::test),
    async(all(not(feature="__test__is_sync"), feature = "__test__tokio"), tokio::test)
)]
async fn test_async_fn() {
    let res = async_fn().await;
    assert_eq!(res, true);
}

#[maybe_async_cfg::maybe(sync(feature = "__test__is_sync", test), async(not(feature = "__test__is_sync"), async_std::test))]
async fn test_async_fn2() {
    let res = async_fn().await;
    assert_eq!(res, true);
}

#[maybe_async_cfg::maybe(sync(feature = "__test__is_sync", test), async(not(feature = "__test__is_sync"), async_std::test))]
async fn test_async_fn3() {
    let res = async_fn().await;
    assert_eq!(res, true);
}

#[maybe_async_cfg::maybe(sync(feature = "__test__is_sync", test), async(not(feature = "__test__is_sync"), async_std::test))]
async fn test_async_fn4() {
    let res = async_fn().await;
    assert_eq!(res, true);
}

fn main() {

}
