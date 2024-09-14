#[maybe_async_cfg::maybe(
    sync(feature = "__test__is_sync"),
    async(not(feature = "__test__is_sync"))
)]
#[allow(dead_code)]
trait T {
    /// This is a doctest
    fn f() {
        todo!()
    }
}
