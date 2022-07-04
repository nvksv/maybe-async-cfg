#[maybe_async_cfg::maybe(sync(feature = "is_sync"), async(not(feature = "is_sync")))]
trait T {
    /// This is a doctest
    fn f() {
        todo!()
    }
}