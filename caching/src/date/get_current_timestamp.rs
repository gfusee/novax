#[cfg(not(test))]
mod implementation {
    use std::time::{SystemTime, UNIX_EPOCH};
    use novax::errors::NovaXError;
    use novax::errors::DateError;

    pub(crate) fn get_current_timestamp() -> Result<u64, NovaXError> {
        let start = SystemTime::now();
        let Ok(timestamp) = start.duration_since(UNIX_EPOCH) else { return Err(DateError::UnableToGetCurrentTimestamp.into())};

        Ok(timestamp.as_secs())
    }
}

pub(crate) use implementation::get_current_timestamp;

#[cfg(test)]
pub(crate) use implementation::set_mock_time;

#[cfg(test)]
mod implementation {
    use novax::errors::NovaXError;
    use std::cell::RefCell;

    thread_local! {
        static MOCK_TIME: RefCell<u64> = RefCell::new(0);
    }

    pub(crate) fn get_current_timestamp() -> Result<u64, NovaXError> {
        let mut time: u64 = 0;
        MOCK_TIME.with(|value| {
            time = *value.borrow();
        });

        Ok(time)
    }

    pub(crate) fn set_mock_time(new_time: u64) {
        MOCK_TIME.with(|value| {
            let mut mock_time = value.borrow_mut();
            *mock_time = new_time;
        });
    }
}