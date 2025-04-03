use core::fmt::{Debug, Formatter};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;

pub struct EventQueryResult<T: Send + Sync> {
    pub timestamp: u64,
    pub event: T,
}

impl<T: Serialize + Send + Sync> Serialize for EventQueryResult<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EventQueryResult", 2)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("event", &self.event)?;
        state.end()
    }
}

impl<'de, T: DeserializeOwned + Send + Sync> Deserialize<'de> for EventQueryResult<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EventQueryResultHelper<T> {
            timestamp: u64,
            event: T,
        }

        let helper = EventQueryResultHelper::deserialize(deserializer)?;
        Ok(EventQueryResult {
            timestamp: helper.timestamp,
            event: helper.event,
        })
    }
}

impl<T: Clone + Send + Sync> Clone for EventQueryResult<T> {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            event: self.event.clone(),
        }
    }
}

impl<T: PartialEq + Send + Sync> PartialEq for EventQueryResult<T> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp) && self.event.eq(&other.event)
    }
}

impl<T: Debug + Send + Sync> Debug for EventQueryResult<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EventQueryResult")
            .field("timestamp", &self.timestamp)
            .field("event", &self.event)
            .finish()
    }
}