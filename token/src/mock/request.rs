use async_trait::async_trait;
use http::StatusCode;
use serde::Serialize;
use serde_json::Value;

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

    async fn post<Body>(&self, body: &Body) -> Result<(StatusCode, Option<String>), RequestError>
        where
            Body: Serialize + Send + Sync
    {
        if !self.get_gateway_url().starts_with(MOCK_BASE_URL) {
            panic!("Url should start with mocked base");
        }

        let serialized = serde_json::to_string(body).unwrap();
        let decoded = serde_json::from_str::<Value>(&serialized).unwrap();

        if let Some((status, data)) = token::get_token_properties_vm_query_response(&decoded) {
            Ok((status, Some(data)))
        } else {
            unreachable!()
        }
    }
}

mod token {
    use hyper::StatusCode;
    use serde_json::Value;

    pub fn get_token_properties_vm_query_response(json: &Value) -> Option<(StatusCode, String)> {
        let sc_address = json.get("scAddress")?;
        let sc_address = sc_address.as_str()?;

        let func_name = json.get("funcName")?;
        let func_name = func_name.as_str()?;

        let args = json.get("args")?;
        let args = args.as_array()?;

        let args: Vec<String> = args
            .iter()
            .map(|e| e.as_str().unwrap().to_string())
            .collect();

        if sc_address != "erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u" || func_name != "getTokenProperties" {
            return None
        }

        if args == ["5745474c442d643763366262"] { //WEGLD-d7c6bb
            Some(get_wegld_gateway_infos())
        } else if args == ["41564153482d313666353330"] { // AVASH-16f530
            Some(get_avash_meta_esdt_gateway_infos())
        } else if args == ["5745474c442d61"] { // WEGLD-a
            Some(get_token_invalid_identifier_infos())
        } else if args == ["5745474c442d616263646566"] { // WEGLD-abcdef
            Some(get_token_not_found_infos())
        } else {
            None
        }
    }

    fn get_wegld_gateway_infos() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"
        {
  "data": {
    "data": {
      "returnData": [
        "V3JhcHBlZEVHTEQ=",
        "RnVuZ2libGVFU0RU",
        "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOs=",
        "MA==",
        "MA==",
        "TnVtRGVjaW1hbHMtMTg=",
        "SXNQYXVzZWQtZmFsc2U=",
        "Q2FuVXBncmFkZS10cnVl",
        "Q2FuTWludC10cnVl",
        "Q2FuQnVybi10cnVl",
        "Q2FuQ2hhbmdlT3duZXItdHJ1ZQ==",
        "Q2FuUGF1c2UtdHJ1ZQ==",
        "Q2FuRnJlZXplLXRydWU=",
        "Q2FuV2lwZS10cnVl",
        "Q2FuQWRkU3BlY2lhbFJvbGVzLXRydWU=",
        "Q2FuVHJhbnNmZXJORlRDcmVhdGVSb2xlLWZhbHNl",
        "TkZUQ3JlYXRlU3RvcHBlZC1mYWxzZQ==",
        "TnVtV2lwZWQtMA=="
      ],
      "returnCode": "ok",
      "returnMessage": "",
      "gasRemaining": 18446744073659552000,
      "gasRefund": 0,
      "outputAccounts": {
        "000000000000000000010000000000000000000000000000000000000002ffff": {
          "address": "erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u",
          "nonce": 0,
          "balance": null,
          "balanceDelta": 0,
          "storageUpdates": {},
          "code": null,
          "codeMetaData": null,
          "outputTransfers": [],
          "callType": 0
        }
      },
      "deletedAccounts": null,
      "touchedAccounts": null,
      "logs": []
    }
  },
  "error": "",
  "code": "successful"
}
        "#.to_string();

        (status, data)
    }

    fn get_avash_meta_esdt_gateway_infos() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"{"data":{"data":{"returnData":["QVZBU0g=","TWV0YUVTRFQ=","AAAAAAAAAAAFAO9UhWcsiU2duCPIgIjJ+/2W7gwiBGM=","MA==","MA==","TnVtRGVjaW1hbHMtMTg=","SXNQYXVzZWQtZmFsc2U=","Q2FuVXBncmFkZS1mYWxzZQ==","Q2FuTWludC1mYWxzZQ==","Q2FuQnVybi1mYWxzZQ==","Q2FuQ2hhbmdlT3duZXItZmFsc2U=","Q2FuUGF1c2UtZmFsc2U=","Q2FuRnJlZXplLWZhbHNl","Q2FuV2lwZS1mYWxzZQ==","Q2FuQWRkU3BlY2lhbFJvbGVzLXRydWU=","Q2FuVHJhbnNmZXJORlRDcmVhdGVSb2xlLWZhbHNl","TkZUQ3JlYXRlU3RvcHBlZC1mYWxzZQ==","TnVtV2lwZWQtMA=="],"returnCode":"ok","returnMessage":"","gasRemaining":18446744073659551615,"gasRefund":0,"outputAccounts":{"000000000000000000010000000000000000000000000000000000000002ffff":{"address":"erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u","nonce":0,"balance":null,"balanceDelta":0,"storageUpdates":{},"code":null,"codeMetaData":null,"outputTransfers":[],"callType":0}},"deletedAccounts":null,"touchedAccounts":null,"logs":[]}},"error":"","code":"successful"}"#.to_string();

        (status, data)
    }
    fn get_token_not_found_infos() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"{"data":{"data":{"returnData":null,"returnCode":"user error","returnMessage":"no ticker with given name","gasRemaining":0,"gasRefund":0,"outputAccounts":{},"deletedAccounts":null,"touchedAccounts":null,"logs":[]}},"error":"","code":"successful"}"#.to_string();

        (status, data)
    }

    fn get_token_invalid_identifier_infos() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"{"data":{"data":{"returnData":null,"returnCode":"user error","returnMessage":"no ticker with given name","gasRemaining":0,"gasRefund":0,"outputAccounts":{},"deletedAccounts":null,"touchedAccounts":null,"logs":[]}},"error":"","code":"successful"}"#.to_string();

        (status, data)
    }
}

mod account {
    use hyper::StatusCode;

    pub fn get_account_response(url: &str) -> Option<(StatusCode, String)> {
        if url.ends_with("/address/erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g/esdt") {
            Some(get_fusee_all_esdts())
        } else if url.ends_with("/address/erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5/esdt") {
            Some(get_invalid_address_all_esdts())
        } else {
            None
        }
    }

    fn get_fusee_all_esdts() -> (StatusCode, String) {
        let status = StatusCode::OK;
        let data = r#"{"data":{"blockInfo":{"hash":"a83c079d78e1ab32e1f4ab4bf286a4b8e4923c7eaf6c4864a28f1bc64ddf04d7","nonce":7626126,"rootHash":"3cb204b521ac54fb9e1bb7dbc26a0f4d10a76abd543445b3f597da2277ede33e"},"esdts":{"ALP-0e6b1c":{"balance":"1696079066943873282","tokenIdentifier":"ALP-0e6b1c"},"ALP-44bcf0":{"balance":"128033502543521903919","tokenIdentifier":"ALP-44bcf0"},"ALP-6b7c94":{"balance":"967770512112525315","tokenIdentifier":"ALP-6b7c94"},"ALP-9b7a73":{"balance":"23307539935215879211","tokenIdentifier":"ALP-9b7a73"},"ALP-a3a2f6":{"balance":"17826305637907730933","tokenIdentifier":"ALP-a3a2f6"},"ALP-fc47a2":{"balance":"23927754058415081874153","tokenIdentifier":"ALP-fc47a2"},"ASH-77a5df":{"balance":"1500470747499707813336","tokenIdentifier":"ASH-77a5df"},"ATS-e57f90":{"balance":"100001113233963657180973456465","tokenIdentifier":"ATS-e57f90"},"AVASH-154d21-01":{"attributes":"AAAAAAAAAAAAAAAJdo6sXgiUqd2A","balance":"1000000","creator":"erd1qqqqqqqqqqqqqpgqwsargntjx2a2knc6r5rrn55dn9ltz62nq33srkuq35","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-154d21-01","uris":[""]},"AVASH-16f530-03":{"attributes":"AAAABQ8Jwd0jAAAAAAAAAAl2XMz7FvZjWAw=","balance":"2183402796320506992652","creator":"erd1qqqqqqqqqqqqqpgqaa2g2eev39xemwprezqg3j0mlktwurpzq33sd3j48g","nonce":3,"royalties":"0","tokenIdentifier":"AVASH-16f530-03","uris":[""]},"AVASH-1f998f-01":{"attributes":"AAAAAAAAAAAAAAAJDH8TBYa8qj6A","balance":"7597456291922700","creator":"erd1qqqqqqqqqqqqqpgq49wfg27l3cgsexw2fy42f2ep0c4402mlq33sqpm303","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-1f998f-01","uris":[""]},"AVASH-1f998f-02":{"attributes":"AAAAAAAAAAAAAAAIGPricK1otR8=","balance":"1040248682055598716","creator":"erd1qqqqqqqqqqqqqpgq49wfg27l3cgsexw2fy42f2ep0c4402mlq33sqpm303","nonce":2,"royalties":"0","tokenIdentifier":"AVASH-1f998f-02","uris":[""]},"AVASH-2fa3be-01":{"attributes":"AAAAAAAAAAAAAAAJg7r4aHtP2S8A","balance":"10000000","creator":"erd1qqqqqqqqqqqqqpgq8evsnerfykptw6h8tcfzxuupgu3dk57kq33shhds8y","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-2fa3be-01","uris":[""]},"AVASH-3e1492-01":{"attributes":"AAAAAAAAAAAAAAAJg7r4aHtPxVjg","balance":"996171652744755363","creator":"erd1qqqqqqqqqqqqqpgqrupe7llr4dqdmwf7f3r3ucwewvmszzx3q33sk3cduv","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-3e1492-01","uris":[""]},"AVASH-3e1492-04":{"attributes":"AAAAAAAAAAAAAAAIfOZAF5uDDeM=","balance":"9951375483997667","creator":"erd1qqqqqqqqqqqqqpgqrupe7llr4dqdmwf7f3r3ucwewvmszzx3q33sk3cduv","nonce":4,"royalties":"0","tokenIdentifier":"AVASH-3e1492-04","uris":[""]},"AVASH-4d18e5-03":{"attributes":"AAAAAAAAAAAAAAAJBczsXRb2zAAA","balance":"107000000000000000000","creator":"erd1qqqqqqqqqqqqqpgqh43pdpehk802zu9cnsm7hzd2p92x99fvq33snxawz5","nonce":3,"royalties":"0","tokenIdentifier":"AVASH-4d18e5-03","uris":[""]},"AVASH-60accf-01":{"attributes":"AAAAAAAAAAAAAAAJDQXU3FlHfA8V","balance":"240227875899485982485","creator":"erd1qqqqqqqqqqqqqpgqsh8ad0zm05z4gulgvrpr3qm9vnzqamv3q33segyh28","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-60accf-01","uris":[""]},"AVASH-7f8164-01":{"attributes":"AAAAAAAAAAAAAAAJgqHK4FVkGxoA","balance":"5000000000","creator":"erd1qqqqqqqqqqqqqpgqkscf0hfys6lq39jpunc7uc0kxm9k9hxzq33s9cy4q8","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-7f8164-01","uris":[""]},"AVASH-8db0f3-01":{"attributes":"AAAAAAAAAAAAAAAJ38fCKcRiWw0A","balance":"79410641","creator":"erd1qqqqqqqqqqqqqpgqwd6tg3gc52jznd99kvyutdcnr6qxzxyzq33sxlccnz","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-8db0f3-01","uris":[""]},"AVASH-8db0f3-03":{"attributes":"AAAAAAAAAAAAAAAJyWR77yAaI79H","balance":"36202562704621383","creator":"erd1qqqqqqqqqqqqqpgqwd6tg3gc52jznd99kvyutdcnr6qxzxyzq33sxlccnz","nonce":3,"royalties":"0","tokenIdentifier":"AVASH-8db0f3-03","uris":[""]},"AVASH-8db0f3-04":{"attributes":"AAAAAAAAAAAAAAAJF7+cMXY7eK+y","balance":"82078636804452274","creator":"erd1qqqqqqqqqqqqqpgqwd6tg3gc52jznd99kvyutdcnr6qxzxyzq33sxlccnz","nonce":4,"royalties":"0","tokenIdentifier":"AVASH-8db0f3-04","uris":[""]},"AVASH-903ea8-79":{"attributes":"AAAAAAAAAAAAAAAJA69bQbbHoYr4","balance":"67975997653935819512","creator":"erd1qqqqqqqqqqqqqpgqvkkdrp0m0rzs4exd6wu8zpkuhzlzw8wvq33sfn3854","nonce":121,"royalties":"0","tokenIdentifier":"AVASH-903ea8-79","uris":[""]},"AVASH-a21c61-01":{"attributes":"AAAAAAAAAAAAAAAJU88NUkDfNt99","balance":"1545999429697017405309","creator":"erd1qqqqqqqqqqqqqpgqwnnm8y24vnlwztnjh4tzsvldv4a23tmjq33s6evxex","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-a21c61-01","uris":[""]},"AVASH-cb0ed8-01":{"attributes":"AAAAAAAAAAAAAAAJarObIW4fYqCA","balance":"200000","creator":"erd1qqqqqqqqqqqqqpgqxyg03lluguyukawha7h7jfxzrrqmd7elq33s86j759","nonce":1,"royalties":"0","tokenIdentifier":"AVASH-cb0ed8-01","uris":[""]},"AVASH-d1701a-07":{"attributes":"AAAABSIvXPyCAAAAAAAAAAmJlMskU3Ep/W4=","balance":"2537925641376893631854","creator":"erd1qqqqqqqqqqqqqpgq695p499ust6xkptf9pmjhmq827f6svkwq33shz73tv","nonce":7,"royalties":"0","tokenIdentifier":"AVASH-d1701a-07","uris":[""]},"AVASH-dc2289-03":{"attributes":"AAAAAAAAAAAAAAAJoYubuzeL+qaA","balance":"2979985635907146000000","creator":"erd1qqqqqqqqqqqqqpgqyq2gfpaftx4ycn36sa9htexmz2k0ygc2q33s8fju0z","nonce":3,"royalties":"0","tokenIdentifier":"AVASH-dc2289-03","uris":[""]},"AVASH-f47572-066a":{"attributes":"AAAABTU2z7I5AAAAAAAAAAmOsy/8Um1/9IU=","balance":"2632349474554405450885","creator":"erd1qqqqqqqqqqqqqpgqematsgxk7xt2e8fr8l8ke6q6jsphypmzq33s57lq6k","nonce":1642,"royalties":"0","tokenIdentifier":"AVASH-f47572-066a","uris":[""]},"AVASH-f47572-066b":{"attributes":"AAAABTU2z7I5AAAAAAAAAAkFaQ4OTks80G4=","balance":"99803724121636655214","creator":"erd1qqqqqqqqqqqqqpgqematsgxk7xt2e8fr8l8ke6q6jsphypmzq33s57lq6k","nonce":1643,"royalties":"0","tokenIdentifier":"AVASH-f47572-066b","uris":[""]},"BTC-be8f22":{"balance":"10011367946","tokenIdentifier":"BTC-be8f22"},"BTE-d634db":{"balance":"74627818032","tokenIdentifier":"BTE-d634db"},"BUSD-632f7d":{"balance":"50402744599274777385","tokenIdentifier":"BUSD-632f7d"},"COPO-e53e1a":{"balance":"7","tokenIdentifier":"COPO-e53e1a"},"EGLDMEX-c29b0e":{"balance":"10080709890705041692","tokenIdentifier":"EGLDMEX-c29b0e"},"EGLDUSDC-842a92":{"balance":"33143062","tokenIdentifier":"EGLDUSDC-842a92"},"ELKMEX-7e6873-0a":{"attributes":"AAAACk1FWC1kYzI4OWMAAAAAAAAAAAAAAAAAAAzG","balance":"2515820157129256605064002","creator":"erd1qqqqqqqqqqqqqpgq97pf6ntww6txw8lccpwszk0a5cfu56dc0n4sknj0p9","name":"MEX-dc289c","nonce":10,"royalties":"0","tokenIdentifier":"ELKMEX-7e6873-0a","uris":[""]},"EVENT-e0b7c1-02":{"attributes":"AAAAAAAAAAIAAABFO21ldGFkYXRhOmJhZmtyZWloNG11c3RremVhZmdiZmhtcmloM2s0c3FpcnU2dWZ0cHd4NGdvamp6dnl3M3Vnem15YjJ1","balance":"1","creator":"erd1qqqqqqqqqqqqqpgqtxpyqrf8fh62r76m2d378kz55q9cp9cted2sstcjfz","name":"Evenement Velodrome #4","nonce":2,"royalties":"100","tokenIdentifier":"EVENT-e0b7c1-02","uris":["aHR0cHM6Ly9pcGZzLmlvL2lwZnMvYmFma3JlaWhsdjNkc2l1b3ZsdXR3YnR0bDdwZGF1ZnN2NWpoenJzZGUydGE1emo2M3dpdWRnM2lydWk=","aHR0cHM6Ly9pcGZzLmlvL2lwZnMvYmFma3JlaWg0bXVzdGt6ZWFmZ2JmaG1yaWgzazRzcWlydTZ1ZnRwd3g0Z29qanp2eXczdWd6bXliMnUg"]},"FARM-c4c5ef-1f52":{"attributes":"AAAABBQU4X0AAAAE7ydxXJ+y2KdDsBjrBTlnPsuT9bwsZTAE/nLafAkBZBViCXHzAAAACA3gtrOnZAAAAAAACA3gtrOnZAAAAAAAAA==","balance":"1000000000000000000","creator":"erd1qqqqqqqqqqqqqpgq84grnrns73p28kkp9mu5v2dl5vclpq6r2gesn5hz4s","nonce":8018,"royalties":"0","tokenIdentifier":"FARM-c4c5ef-1f52","uris":[""]},"FUU-ab98ef-5c":{"attributes":"AAAABiWn6imtwgAAAAYDmuBcYxKfstinQ7AY6wU5Zz7Lk/W8LGUwBP5y2nwJAWQVYglx8wAAAAkEhMb9oJCXWecAAAAJC0aUbr7XIXKz","balance":"41677294809647918324","creator":"erd1qqqqqqqqqqqqqpgq6euk6n5av376vqg768z48zrv4fjxyw63rmcq8cgglq","nonce":92,"royalties":"0","tokenIdentifier":"FUU-ab98ef-5c","uris":[""]},"HTM-fe1f69":{"balance":"51307837643238304845","tokenIdentifier":"HTM-fe1f69"},"LKMEX-3b7d9a-01df":{"attributes":"AAAABgAAAAAAAAzKAAAAAAAAQmgAAAAAAAAM6AAAAAAAAEJoAAAAAAAADQYAAAAAAABCaAAAAAAAAA0kAAAAAAAAQmgAAAAAAAANQgAAAAAAAD6AAAAAAAAADWAAAAAAAAA+gAA=","balance":"5000000000000000000000000","creator":"erd1qqqqqqqqqqqqqpgqnqf6qpnd7y3m6wpkur9u8hg37rvhn5ae0n4se7lw39","nonce":479,"royalties":"0","tokenIdentifier":"LKMEX-3b7d9a-01df","uris":[""]},"LPT-93bf25":{"balance":"323999958827383942267","tokenIdentifier":"LPT-93bf25"},"MEX-dc289c":{"balance":"82518169348059734076078","tokenIdentifier":"MEX-dc289c"},"ONE-8976c8":{"balance":"1553704024939502919","tokenIdentifier":"ONE-8976c8"},"RENBTC-a74396":{"balance":"5000000","tokenIdentifier":"RENBTC-a74396"},"SATS-4308b1-01":{"attributes":"AAAAAAAAAAAAAAAJNjXJrcXeoAAA","balance":"1000000000000000000000","creator":"erd1qqqqqqqqqqqqqpgqwq7j2sghx8xeutjp7yu7ufw7wz4cz8x0q33s0anwu6","nonce":1,"royalties":"0","tokenIdentifier":"SATS-4308b1-01","uris":[""]},"SATS-47c51f-056d":{"attributes":"AAAABA5TFrgAAAAAAAAACgRF2gCcX8gIAAA=","balance":"20178000000000000000000","creator":"erd1qqqqqqqqqqqqqpgq5gge8r9yw3a4ycwufs332840v2h0zqkfq33sg26r2g","nonce":1389,"royalties":"0","tokenIdentifier":"SATS-47c51f-056d","uris":[""]},"SATS-cd53e8-02":{"attributes":"AAAAAAAAAAAAAAAIDeC2s6dkAAA=","balance":"1000000000000000000","creator":"erd1qqqqqqqqqqqqqpgqttgmw8wm3kez444lck5qqvf28sxsc7yrq33sy7hnyl","nonce":2,"royalties":"0","tokenIdentifier":"SATS-cd53e8-02","uris":[""]},"SBATS-d81f46-06":{"attributes":"ZCWh4A==","balance":"5000000000000000000","creator":"erd1qqqqqqqqqqqqqpgq5gge8r9yw3a4ycwufs332840v2h0zqkfq33sg26r2g","nonce":6,"royalties":"0","tokenIdentifier":"SBATS-d81f46-06","uris":[""]},"SBATS-d81f46-09":{"attributes":"ZCXm2g==","balance":"1000000000000000000","creator":"erd1qqqqqqqqqqqqqpgq5gge8r9yw3a4ycwufs332840v2h0zqkfq33sg26r2g","nonce":9,"royalties":"0","tokenIdentifier":"SBATS-d81f46-09","uris":[""]},"TEST-f2591f":{"balance":"99999999999999999999999880000000000000000000","tokenIdentifier":"TEST-f2591f"},"TESTEGLD-ee067b":{"balance":"1000000000000000000","tokenIdentifier":"TESTEGLD-ee067b"},"TOKN-d0a591":{"balance":"100000000000000000000000000000000","tokenIdentifier":"TOKN-d0a591"},"USDC-6c5d88":{"balance":"1171000000","tokenIdentifier":"USDC-6c5d88"},"USDC-8d4068":{"balance":"2147683627","tokenIdentifier":"USDC-8d4068"},"USDT-188935":{"balance":"114625802","tokenIdentifier":"USDT-188935"},"USDT-324eda":{"balance":"297487589","tokenIdentifier":"USDT-324eda"},"WBTC-1297c1":{"balance":"5000000","tokenIdentifier":"WBTC-1297c1"},"WEGLD-d7c6bb":{"balance":"71179029947004300508","tokenIdentifier":"WEGLD-d7c6bb"},"XTICKEVENT-fc3bc6-01":{"attributes":"TmFtZTpKZWFuIC0tLSBNYXJzZWlsbGU7RGF0ZToxNjg3NTQzMjAwO21ldGFkYXRhOmJhZmtyZWlnbGhlNHllMmY0NGFvbmZhYndmYzRtaWFoMnN4eGNnZ3V2bmZ4eDNkdWc1bWhuNnVicnRx","balance":"1","creator":"erd1qqqqqqqqqqqqqpgq22ts36la7j20y9n40namzdrcxy0ww7qlt2tsu7j6v6","name":"Jean --- Marseille #1","nonce":1,"royalties":"1000","tokenIdentifier":"XTICKEVENT-fc3bc6-01","uris":["aHR0cHM6Ly9pcGZzLmlvL2lwZnMvYmFma3JlaWN5dGNpaWkzb3ZubDdmcTJlYWxqcGk0N3N5bWJseHVlcWFmNmwzNnRpN3BtaXdod3l3d3E=","aHR0cHM6Ly9pcGZzLmlvL2lwZnMvYmFma3JlaWdsaGU0eWUyZjQ0YW9uZmFid2ZjNG1pYWgyc3h4Y2dndXZuZnh4M2R1ZzVtaG42dWJydHE="]},"XTICKEVENT-fc3bc6-02":{"attributes":"TmFtZTpKZWFuIC0tLSBNYXJzZWlsbGU7RGF0ZToxNjg3NTQzMjAwO21ldGFkYXRhOmJhZmtyZWlnbGhlNHllMmY0NGFvbmZhYndmYzRtaWFoMnN4eGNnZ3V2bmZ4eDNkdWc1bWhuNnVicnRx","balance":"1","creator":"erd1qqqqqqqqqqqqqpgq22ts36la7j20y9n40namzdrcxy0ww7qlt2tsu7j6v6","name":"Jean --- Marseille #2","nonce":2,"royalties":"1000","tokenIdentifier":"XTICKEVENT-fc3bc6-02","uris":["aHR0cHM6Ly9pcGZzLmlvL2lwZnMvYmFma3JlaWN5dGNpaWkzb3ZubDdmcTJlYWxqcGk0N3N5bWJseHVlcWFmNmwzNnRpN3BtaXdod3l3d3E=","aHR0cHM6Ly9pcGZzLmlvL2lwZnMvYmFma3JlaWdsaGU0eWUyZjQ0YW9uZmFid2ZjNG1pYWgyc3h4Y2dndXZuZnh4M2R1ZzVtaG42dWJydHE="]}}},"error":"","code":"successful"}"#.to_string();

        (status, data)
    }

    fn get_invalid_address_all_esdts() -> (StatusCode, String) {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let data = r#"{"data":null,"error":"cannot get ESDT token data: invalid checksum (expected (bech32=yjvdh0, bech32m=yjvdh03wupjd), got sg88f5)","code":"internal_issue"}"#.to_string();

        (status, data)
    }
}