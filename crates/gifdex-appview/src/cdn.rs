use jacquard_common::types::{cid::Cid, did::Did, string::Rkey};
use reqwest::Url;

pub struct CdnClient {
    base_url: Url,
}

pub enum CdnMediaType<'a> {
    Avatar {
        did: &'a Did<'a>,
        cid: &'a Cid<'a>,
    },
    PostMedia {
        did: &'a Did<'a>,
        rkey: &'a Rkey<'a>,
        thumbnail: bool,
    },
}

impl CdnClient {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    pub fn make_cdn_url(&self, media_type: CdnMediaType) -> Url {
        match media_type {
            CdnMediaType::Avatar { did, cid } => self
                .base_url
                .join(&format!("/avatar/{}/{}", did, cid))
                .expect("avatar url construction should never fail"),
            CdnMediaType::PostMedia {
                did,
                rkey: tid,
                thumbnail,
            } => {
                if thumbnail {
                    self.base_url
                        .join(&format!("/media/{}/{}", did, tid))
                        .expect("media url construction should never fail")
                } else {
                    self.base_url
                        .join(&format!("/media/{}/{}", did, tid))
                        .expect("media url construction should never fail")
                }
            }
        }
    }
}
