use jacquard_common::{
    IntoStatic,
    types::{cid::Cid, did::Did, nsid::Nsid, string::Handle, string::Rkey, tid::Tid},
};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RepoCountResponse {
    pub repo_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RecordCountResponse {
    pub record_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct OutboxBufferResponse {
    pub outbox_buffer: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ResyncBufferResponse {
    pub resync_buffer: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RepoInfo<'a> {
    #[serde(borrow)]
    pub did: Did<'a>,
    #[serde(borrow)]
    pub handle: Handle<'a>,
    pub state: RepoState,
    pub rev: Tid,
    pub error: String,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "action", rename_all = "lowercase")]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
#[non_exhaustive]
pub struct RecordPayload {
    value: serde_json::Value,
}

impl RecordPayload {
    pub fn parse<T: serde::de::DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.value)
    }

    pub fn raw(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn record_type(&self) -> Option<&str> {
        self.value.get("$type")?.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct IdentityEventData<'a> {
    #[serde(borrow)]
    pub did: Did<'a>,
    pub handle: String,
    pub is_active: bool,
    pub status: String,
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
