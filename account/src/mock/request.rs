use async_trait::async_trait;
use http::StatusCode;
use serde::Serialize;

use novax_request::error::request::RequestError;
use novax_request::gateway::client::GatewayClient;

const MOCK_BASE_URL: &str = "https://test.test";

pub struct MockClient {
    url: String
}

impl MockClient {
    pub fn new() -> MockClient {
        MockClient {
            url: MOCK_BASE_URL.to_string(),
        }
    }
}

#[async_trait]
impl GatewayClient for MockClient {
    type Owned = Self;

    fn get_gateway_url(&self) -> &str {
        &self.url
    }

    fn with_appended_url(&self, url: &str) -> Self {
        MockClient {
            url: format!("{}{url}", self.url),
        }
    }

    async fn get(&self) -> Result<(StatusCode, Option<String>), RequestError> {
        if let Some((status, data)) = account::get_account_response(&self.url) {
            Ok((status, Some(data)))
        } else {
            panic!("Unknown url: {}", self.url)
        }
    }

    async fn post<Body>(&self, _: &Body) -> Result<(StatusCode, Option<String>), RequestError>
        where
            Body: Serialize + Send + Sync
    {
        unreachable!()
    }
}

mod account {
    use hyper::StatusCode;

    pub fn get_account_response(url: &str) -> Option<(StatusCode, String)> {
        if url.ends_with("/address/erd1qqqqqqqqqqqqqpgqr7een4m5z44frr3k35yjdjcrfe6703cwdl3s3wkddz") {
            Some(get_xportal_xp_sc_account())
        } else if url.ends_with("/address/erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd") {
            Some(get_xportal_xp_sc_owner_account())
        } else if url.ends_with("/address/erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5") {
            Some(get_invalid_address_account())
        } else {
            None
        }
    }

    fn get_xportal_xp_sc_account() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"{
                                "data": {
                                    "account": {
                                        "address": "erd1qqqqqqqqqqqqqpgqr7een4m5z44frr3k35yjdjcrfe6703cwdl3s3wkddz",
                                        "nonce": 0,
                                        "balance": "0",
                                        "username": "",
                                        "code": "fakecodestring",
                                        "codeHash": "gVgRRf6HhmTGlxziasAFoCgBlP7/DH0i9IhTbj7lsxA=",
                                        "rootHash": "A3RZ7aYh4NzkunNL+fu09ggnItEeC7SuPWJDfIHmAcI=",
                                        "codeMetadata": "BQA=",
                                        "developerReward": "2288888045322000000",
                                        "ownerAddress": "erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd"
                                    },
                                    "blockInfo": {
                                        "nonce": 20872513,
                                        "hash": "ff87feddcfee21387d28b4e95685987743d8028e8c92b13338b6129d7591ed53",
                                        "rootHash": "3574f6febada139f25bdc6293fdf70366cbf05bc4c2592a7454484b709c695c0"
                                    }
                                },
                                "error": "",
                                "code": "successful"
                            }"#.to_string();

        (status, data)
    }

    fn get_xportal_xp_sc_owner_account() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"{
                                "data": {
                                    "account": {
                                        "address": "erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd",
                                        "nonce": 6,
                                        "balance": "412198271210000000",
                                        "username": "",
                                        "code": "",
                                        "codeHash": null,
                                        "rootHash": "Juj3aJQOKv4DzZG3XOueG934NL7pq/7bmiVnR4zzXAo=",
                                        "codeMetadata": null,
                                        "developerReward": "0",
                                        "ownerAddress": ""
                                    },
                                    "blockInfo": {
                                        "nonce": 20872528,
                                        "hash": "4df35bf47c18c1211fc869953091f82e6b1cc3900d5c8f75db964fe77dac8512",
                                        "rootHash": "1bedc08dfd779fdd7f6a43db46a68533307f7a08ca9871f836816b9992cb0bf1"
                                    }
                                },
                                "error": "",
                                "code": "successful"
                            }"#.to_string();

        (status, data)
    }

    fn get_invalid_address_account() -> (StatusCode, String) {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let data = r#"{
                                "data": null,
                                "error": "cannot get account: invalid checksum (expected (bech32=mxv7tl, bech32m=mxv7tlw6ujwa), got 85klfd)",
                                "code": "internal_issue"
                            }"#.to_string();

        (status, data)
    }
}