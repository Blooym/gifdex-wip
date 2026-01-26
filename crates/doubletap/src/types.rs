use jacquard_common::{
    IntoStatic,
    bytes::Bytes,
    types::{
        cid::Cid,
        did::Did,
        nsid::Nsid,
        string::{Handle, Rkey},
        tid::Tid,
    },
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, hash::Hash};

// HTTP Request/Response types

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[non_exhaustive]
pub struct RepoCountResponse {
    pub repo_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[non_exhaustive]
pub struct RecordCountResponse {
    pub record_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[non_exhaustive]
pub struct OutboxBufferResponse {
    pub outbox_buffer: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[non_exhaustive]
pub struct ResyncBufferResponse {
    pub resync_buffer: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct CursorsResponse {
    pub firehose: Option<u64>,
    pub list_repos: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum RepoState {
    Pending,
    Desynchronized,
    Resyncing,
    Active,
    Takendown,
    Suspended,
    Deactivated,
    Error,
}

impl RepoState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Desynchronized => "desynchronized",
            Self::Resyncing => "resyncing",
            Self::Active => "active",
            Self::Takendown => "takendown",
            Self::Suspended => "suspended",
            Self::Deactivated => "deactivated",
            Self::Error => "error",
        }
    }
}

impl Display for RepoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct RepoInfo<'a> {
    #[serde(borrow)]
    pub did: Did<'a>,
    #[serde(borrow)]
    pub handle: Handle<'a>,
    pub state: RepoState,
    pub rev: Tid,
    pub error: Box<str>,
    pub retries: u64,
    pub records: u64,
}

impl IntoStatic for RepoInfo<'_> {
    type Output = RepoInfo<'static>;
    fn into_static(self) -> Self::Output {
        RepoInfo {
            did: self.did.into_static(),
            handle: self.handle.into_static(),
            state: self.state,
            rev: self.rev,
            error: self.error,
            retries: self.retries,
            records: self.records,
        }
    }
}

// WS Channel Types

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct Event<'a> {
    pub id: u64,
    #[serde(flatten, borrow)]
    pub data: EventData<'a>,
}

impl IntoStatic for Event<'_> {
    type Output = Event<'static>;
    fn into_static(self) -> Self::Output {
        Event {
            id: self.id,
            data: self.data.into_static(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum EventData<'a> {
    Record {
        #[serde(borrow)]
        record: RecordEventData<'a>,
    },
    Identity {
        #[serde(borrow)]
        identity: IdentityEventData<'a>,
    },
}

impl IntoStatic for EventData<'_> {
    type Output = EventData<'static>;
    fn into_static(self) -> Self::Output {
        match self {
            EventData::Record { record } => EventData::Record {
                record: record.into_static(),
            },
            EventData::Identity { identity } => EventData::Identity {
                identity: identity.into_static(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct RecordEventData<'a> {
    pub live: bool,
    #[serde(borrow)]
    pub did: Did<'a>,
    pub rev: Tid,
    #[serde(borrow)]
    pub collection: Nsid<'a>,
    #[serde(borrow)]
    pub rkey: Rkey<'a>,
    #[serde(flatten, borrow)]
    pub action: RecordAction<'a>,
}

impl IntoStatic for RecordEventData<'_> {
    type Output = RecordEventData<'static>;
    fn into_static(self) -> Self::Output {
        RecordEventData {
            live: self.live,
            did: self.did.into_static(),
            rev: self.rev,
            collection: self.collection.into_static(),
            rkey: self.rkey.into_static(),
            action: self.action.into_static(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action", rename_all = "lowercase")]
#[non_exhaustive]
pub enum RecordAction<'a> {
    Create {
        record: Box<RecordPayload>,
        #[serde(borrow)]
        cid: Cid<'a>,
    },
    Update {
        record: Box<RecordPayload>,
        #[serde(borrow)]
        cid: Cid<'a>,
    },
    Delete,
}

impl IntoStatic for RecordAction<'_> {
    type Output = RecordAction<'static>;
    fn into_static(self) -> Self::Output {
        match self {
            RecordAction::Create { record, cid } => RecordAction::Create {
                record,
                cid: cid.into_static(),
            },
            RecordAction::Update { record, cid } => RecordAction::Update {
                record,
                cid: cid.into_static(),
            },
            RecordAction::Delete => RecordAction::Delete,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RecordPayload {
    value: Bytes,
}

// TODO: This is a hack so we can support deserializing non-owned types.
// In the future the channel code will be rewritten to support more efficient processing
// and have less overhead.

impl<'de> Deserialize<'de> for RecordPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        Ok(RecordPayload {
            value: Bytes::from(
                serde_json::to_vec(&serde_json::Value::deserialize(deserializer)?)
                    .map_err(Error::custom)?,
            ),
        })
    }
}

impl Serialize for RecordPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        serde_json::from_slice::<serde_json::Value>(&self.value)
            .map_err(Error::custom)?
            .serialize(serializer)
    }
}

impl RecordPayload {
    /// Deserialize the record payload into a given type.
    pub fn deserialize<'de, T: serde::de::Deserialize<'de>>(
        &'de self,
    ) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IdentityEventStatus {
    Active,
    Takendown,
    Suspended,
    Deactivated,
    Deleted,
}

impl IdentityEventStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Takendown => "takendown",
            Self::Suspended => "suspended",
            Self::Deactivated => "deactivated",
            Self::Deleted => "deleted",
        }
    }
}

impl Display for IdentityEventStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct IdentityEventData<'a> {
    #[serde(borrow)]
    pub did: Did<'a>,
    pub handle: String,
    pub is_active: bool,
    pub status: IdentityEventStatus,
}

impl IntoStatic for IdentityEventData<'_> {
    type Output = IdentityEventData<'static>;
    fn into_static(self) -> Self::Output {
        IdentityEventData {
            did: self.did.into_static(),
            handle: self.handle,
            is_active: self.is_active,
            status: self.status,
        }
    }
}
