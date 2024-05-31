use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponse {
    pub data: Option<VmValuesQueryResponseData>,
    pub error: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponseData {
    pub data: VmValuesQueryResponseDataData
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponseDataData {
    pub return_data: Option<Vec<String>>,
    pub return_message: String
}

#[cfg(test)]
mod tests {
    use crate::network::query::models::response::VmValuesQueryResponse;

    #[test]
    fn test_deserialize_execution_error() {
        let data = r#"
{
  "data": {
    "blockInfo": {
      "nonce": 20144493,
      "hash": "325c96bb497e497fe417aa20dbcba9fc233654e9999982fdf9f5599ec83daa2f",
      "rootHash": "81e0c2bcb292ae083af60fdfa1889d032dbfc79e4d14a8d1a0dbe9c5606d5033"
    },
    "data": {
      "returnData": null,
      "returnCode": "contract not found",
      "returnMessage": "invalid contract code (not found)",
      "gasRemaining": 0,
      "gasRefund": 0,
      "outputAccounts": {},
      "deletedAccounts": null,
      "touchedAccounts": null,
      "logs": [
        {
          "identifier": "aW50ZXJuYWxWTUVycm9ycw==",
          "address": "erd1qqqqqqqqqqqqqpgqhnmuen6gx7unfmqsjwx0ul7ezjyg2ndfvcqsa4nqax",
          "topics": [
            "AAAAAAAAAAAFALz3zM9IN7k07BCTjP5/2RSIhU2pZgE=",
            "dmlld1BhaXJz"
          ],
          "data": "CglydW50aW1lLmdvOjgyNyBbaW52YWxpZCBjb250cmFjdCBjb2RlIChub3QgZm91bmQpXSBbdmlld1BhaXJzXQ==",
          "additionalData": [
            "CglydW50aW1lLmdvOjgyNyBbaW52YWxpZCBjb250cmFjdCBjb2RlIChub3QgZm91bmQpXSBbdmlld1BhaXJzXQ=="
          ]
        }
      ]
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        _ = serde_json::from_str::<VmValuesQueryResponse>(data).unwrap();
    }
}