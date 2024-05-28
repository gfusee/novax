pub(crate) fn get_timestamp_of_next_block(current_timestamp: Duration) -> Result<Duration, NovaXError> {
    let mut timestamp = current_timestamp.as_secs() + 1;
    while timestamp % 6 != 0 {
        timestamp += 1
    }

    Ok(Duration::from_secs(timestamp))
}

#[cfg(not(test))]
mod implementation {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use novax::errors::NovaXError;
    use novax::errors::DateError;

    pub(crate) fn get_current_timestamp() -> Result<Duration, NovaXError> {
        let start = SystemTime::now();
        let Ok(timestamp) = start.duration_since(UNIX_EPOCH) else { return Err(DateError::UnableToGetCurrentTimestamp.into())};

        Ok(timestamp)
    }
}

use std::time::Duration;
pub(crate) use implementation::get_current_timestamp;

#[cfg(test)]
pub(crate) use implementation::set_mock_time;
use novax::errors::NovaXError;

#[cfg(test)]
mod implementation {
    use novax::errors::NovaXError;
    use std::cell::RefCell;
    use core::time::Duration;

    thread_local! {
        static MOCK_TIME: RefCell<Duration> = RefCell::new(Duration::from_secs(0));
    }

    pub(crate) fn get_current_timestamp() -> Result<Duration, NovaXError> {
        let mut time: Duration = Duration::from_secs(0);
        MOCK_TIME.with(|value| {
            time = *value.borrow();
        });

        Ok(time)
    }

    pub(crate) fn set_mock_time(new_time: Duration) {
        MOCK_TIME.with(|value| {
            let mut mock_time = value.borrow_mut();
            *mock_time = new_time;
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::date::get_current_timestamp::get_timestamp_of_next_block;

    #[test]
    fn test_get_timestamp_of_next_block_start_of_block() {
        let result = get_timestamp_of_next_block(Duration::from_secs(6)).unwrap();
        let expected = Duration::from_secs(12);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_timestamp_of_next_block_mid_of_block() {
        let result = get_timestamp_of_next_block(Duration::from_secs(8)).unwrap();
        let expected = Duration::from_secs(12);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_timestamp_of_next_block_end_of_block() {
        let result = get_timestamp_of_next_block(Duration::from_secs(11)).unwrap();
        let expected = Duration::from_secs(12);

        assert_eq!(result, expected);
    }
}