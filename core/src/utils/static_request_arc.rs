use std::sync::{Arc, Mutex};
use multiversx_sc_scenario::api::StaticApi;

thread_local! {
    /// Useful to know when we can call `StaticApi::reset()` safely.
    /// Each request will clone this value, increasing the strong references count by one.
    ///
    /// After each request, the cloned value will be released, implying the reference count to decrease by one.
    ///
    /// With this mechanism, we can ensure that a call to `StaticApi::reset()` is safe is the strong references count is 1 (the one in this file)
    /// Otherwise, calling result would lead to bad behaviors for ongoing requests that have ManagedTypes instantiated.
    static STATIC_REQUEST_ARC: StaticRequestArcWrapper = StaticRequestArcWrapper::default();
}

#[derive(Clone, Default)]
pub struct StaticRequestArcWrapper {
    /// Locked only while the API is being reset.
    /// So no new request is launched while the reset.
    locker: Arc<Mutex<()>>,
    /// If the count of arc is 1 there is no ongoing request.
    /// It means that the API can be safely reset.
    arc: Arc<()>
}

impl Drop for StaticRequestArcWrapper {
    fn drop(&mut self) {
        // The arc is only dropped after the drop function.
        // Therefore, we have to anticipate.
        let arc_strong_count_after_drop = Arc::strong_count(&self.arc) - 1;
        if arc_strong_count_after_drop == 1 {
            let guard = self.locker.lock().unwrap(); // Ensure no new request is initiated
            StaticApi::reset();
            drop(guard);
        }
    }
}

pub fn get_static_request_arc_clone() -> StaticRequestArcWrapper {
    STATIC_REQUEST_ARC.with(|e| {
        // First we don't want to perform any operation if the API is being reset
        let guard = e.locker.lock().unwrap();
        drop(guard);

        e.clone()
    })
}
