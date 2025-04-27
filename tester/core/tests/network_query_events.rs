use novax::tester::tester::{EventWithMultiValueEncodedEventQueryResult, EventWithOnlyData};
use novax::tester::tester::EventWithOnlyDataEventQueryResult;
use async_trait::async_trait;
use novax::errors::NovaXError;
use novax::executor::{BaseElasticSearchNodeQueryExecutor, ElasticSearchClient, ElasticSearchNodeProxy, EventQueryOptions, EventQueryResult, ExecutorError, QueryEventsSortOptions, SortOption, TimestampOption};
use novax::pair::pair::{PairContract, SwapEvent, SwapEventFilterOptions, SwapEventQueryResult};
use novax::Address;
use num_bigint::BigUint;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use novax::tester::tester::{EmptyEventEventQueryResult, TesterContract};

const TESTER_POOL_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"; // This is an xExchange LP contract on mainnet.
const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c"; // This is the tester contrat on devnet.

#[derive(Clone)]
struct MockElasticSearchClient;

#[async_trait]
impl ElasticSearchClient for MockElasticSearchClient {
    fn new(_gateway_url: String) -> Self {
        MockElasticSearchClient
    }

    async fn search(&self, index: &str, query_body: Value) -> Result<Value, ExecutorError> {
        if index != "events" {
            panic!("Unexpected index: {}", index);
        };

        println!("{}", query_body.to_string());

        let known_requests = [
            Self::query_swaps_no_filter_no_options(),
            Self::query_swaps_no_filter_with_size_option(),
            Self::query_swaps_no_filter_with_from_option(),
            Self::query_swaps_no_filter_with_gte_timestamp_option(),
            Self::query_swaps_no_filter_with_lte_timestamp_option(),
            Self::query_swaps_no_filter_with_between_timestamp_option(),
            Self::query_swaps_no_filter_with_sort_timestamp_ascending_option(),
            Self::query_swaps_no_filter_with_sort_timestamp_descending_option(),
            Self::query_swaps_with_one_field_filter_no_options(),
            Self::query_swaps_with_two_fields_filter_no_options(),
            Self::query_swaps_with_all_fields_filter_and_all_options(),
            Self::query_empty_events_no_filter_no_options(),
            Self::query_events_with_only_data_no_filter_no_options(),
            Self::query_events_with_multi_value_encoded_and_data_no_filter_no_options()
        ];

        for (body, response) in known_requests.iter() {
            if query_body == *body {
                return Ok(response.clone());
            }
        }

        panic!("Unexpected query: {:?}", query_body);
    }
}

impl MockElasticSearchClient {
    fn query_swaps_no_filter_no_options() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}}]}}}"#;

        let response = r#"{"took":23,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":10000,"relation":"gte"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"8d08b40394c2af86e88c781e6be94c197afffcd8657c9ee1edd0093891e2c353-1-5","_score":0.0,"_source":{"logAddress":"erd1l7f7qeppj39famz0d4s6dlzzqk8v53hjrprqd5v2ykkprfxny26sxy0rqk","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d61323863353900000008135f726e04c2fe000000000b555344432d333530633465000000040276a0780000000704f59e5169aac00000000a03163fcb092f45061ecf0000000564c7375403000000000010208d00000000000001b800000000655940f6","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01b8"],"shardID":1,"txOrder":0,"txHash":"8d08b40394c2af86e88c781e6be94c197afffcd8657c9ee1edd0093891e2c353","uuid":"U_Jhs8Htz7cRudrSk2RREg","order":5,"timestamp":1700348150}},{"_index":"events-000001","_id":"08c1276fdfa3cb1f91813252cdfacda3ed83f5905de6cebcae9e420b598d82db-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"a322a8114c812a57330417558b9822b4a3c287d4a22ec3888136fe82df93ea9f0000000c5745474c442d613238633539000000080de0b6b3a76400000000000b555344432d3335306334650000000401c2e8f200000007038d7ea4c680000000000a0316f7f85eb32d5fc3d70000000564af5001d200000000000fd2af00000000000001b00000000065576dc2","topics":["73776170","5745474c442d613238633539","555344432d333530633465","a322a8114c812a57330417558b9822b4a3c287d4a22ec3888136fe82df93ea9f","01b0"],"shardID":1,"txOrder":0,"txHash":"08c1276fdfa3cb1f91813252cdfacda3ed83f5905de6cebcae9e420b598d82db","uuid":"1M1MCZJiJHmw6Iz7YGlnvA","order":4,"timestamp":1700228546}},{"_index":"events-000001","_id":"2499d027cc66c1b14a78a2abc0c29d282c5309298347aed68c0484966ba9cdda-1-8","_score":0.0,"_source":{"originalTxHash":"43fbc29510c46ccd779063bb91f2f15ea7000bff377777624fe2b287cddf062d","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000091576d00774744da5200000000b555344432d3335306334650000000502cb85edbe00000008057eac0a19b75fa70000000a0317bc19fe36d1cf963000000005649b9081d80000000000126e2600000000000001f700000000656714c6","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"2499d027cc66c1b14a78a2abc0c29d282c5309298347aed68c0484966ba9cdda","uuid":"fKNQBSJ3l6uLdRxKIeShzg","order":8,"timestamp":1701254342}},{"_index":"events-000001","_id":"4d4351c3446b871485389ad09fd1aba38aea108dcccdda64b19002822c8dfc74-1-5","_score":0.0,"_source":{"originalTxHash":"b92300a77becad553eacf6d1edfecbbbef8d3e3eb7a7a0bbb1c4048b37d91206","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000b555344432d3335306334650000000502cb854d3e0000000c5745474c442d6132386335390000000915569d37b474aca52200000003b72c5d0000000567665ea2b90000000a0302657cc6825d22f10e0000000000126e3900000000000001f70000000065671538","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"4d4351c3446b871485389ad09fd1aba38aea108dcccdda64b19002822c8dfc74","uuid":"a6hOW6GSerN-p8i6_rVxSg","order":5,"timestamp":1701254456}},{"_index":"events-000001","_id":"8d7e6894f32273bac9be662684c227ed2c5a9d995307610ea10e6f0d0e77f4f3-1-8","_score":0.0,"_source":{"originalTxHash":"85e9d026f21a2866487d89e5cc6fde28798b5ec0e737978ad9d50b59a496630b","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000093a737cf4ab84f703330000000b555344432d3335306334650000000507458271f9000000080ef6a7281b8664190000000a033cca031405c6939028000000056020dc30c00000000000126e9f00000000000001f7000000006567179c","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"8d7e6894f32273bac9be662684c227ed2c5a9d995307610ea10e6f0d0e77f4f3","uuid":"n38RlkgkE_Cr73NISJmOSA","order":8,"timestamp":1701255068}},{"_index":"events-000001","_id":"ba62baa239eed1795556891681f9fb0eee79d676868af297c94a2246eef2c47e-1-5","_score":0.0,"_source":{"originalTxHash":"c963399f91bfe82d15a51d5b385c4ddbe4dac4171fd1ef1c936ee83ba2f66362","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000b555344432d33353063346500000005074393faf60000000c5745474c442d613238633539000000093a10b8e8a9b8b558c00000000401dc0d4e000000056762941e680000000a0302b94a2b5c0dde37680000000000126eaf00000000000001f700000000656717fc","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"ba62baa239eed1795556891681f9fb0eee79d676868af297c94a2246eef2c47e","uuid":"VI7k8Md_ORN6yF-PcQ-FHw","order":5,"timestamp":1701255164}},{"_index":"events-000001","_id":"ee638ffc03c26a1a9fd0d81d30f9a3348f1433a4d9864cc02340fd11f448c239-1-8","_score":0.0,"_source":{"originalTxHash":"89be3e76c48b1924bcb5422020a293c124c877cb12f912501255a18f5b2c651e","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000091308e7a5bf4f3402c40000000b555344432d3335306334650000000501c00eb2ec0000000804df76b1996c570a0000000a03ab3da41c4ac46ece7f0000000554de3da8310000000000126f8700000000000001f70000000065671d0c","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"ee638ffc03c26a1a9fd0d81d30f9a3348f1433a4d9864cc02340fd11f448c239","uuid":"1SjN0OgAKBWslxfY0JyjNw","order":8,"timestamp":1701256460}},{"_index":"events-000001","_id":"f051300339b955f01cbb9376dd3912cb5b4e1b7154c01c71f1d6bab838864306-1-5","_score":0.0,"_source":{"logAddress":"erd1f0dwxpl3vxe936cla2mkky7nym4g3xn4vgfz497dpupqul8uktzshxqj5l","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000080ddf9e81049e737f0000000b555344432d33353063346500000004013f942b00000007038d36e9bb8cea0000000a03abd24bb2fc19cc47670000000554d0d805b9000000000012710800000000000001f70000000065672612","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"f051300339b955f01cbb9376dd3912cb5b4e1b7154c01c71f1d6bab838864306","uuid":"QJwqgkwyOdOsG8DZtBEo1g","order":5,"timestamp":1701258770}},{"_index":"events-000001","_id":"05b8bb479cee9990e3bd7d516d086ff5b164598717057b7e57882d261de7b013-1-7","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqzshqdqcdzdl43vhy7p7q8uhc5xzu5x7zh2usyz5kg6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab90000000c5745474c442d61323863353900000008016345785d8a00000000000b555344432d333530633465000000031ff738000000065af3107a40000000000a03abd3ae9d8166dc07670000000554d0b80e8100000000001271d700000000000001f70000000065672aec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab9","01f7"],"shardID":1,"txOrder":0,"txHash":"05b8bb479cee9990e3bd7d516d086ff5b164598717057b7e57882d261de7b013","uuid":"iPeFxf4DPXWFmHxfbZqTQA","order":7,"timestamp":1701260012}},{"_index":"events-000001","_id":"c8786e105d5ff373fe6bb4adc08322e59c180a883682afcae2ed8dd717885ed8-1-27","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqarg523n0ja32xq6nr3dech7a4aq4xeyg5zvsfwl04y","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb0000000b555344432d3335306334650000000206180000000c5745474c442d6132386335390000000643520e2b68ba00000001010000000554d0ddcded0000000a03abd20d2da3f989395c000000000012735700000000000001f800000000656733ec","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb","01f8"],"shardID":1,"txOrder":0,"txHash":"c8786e105d5ff373fe6bb4adc08322e59c180a883682afcae2ed8dd717885ed8","uuid":"9pg7ZIVN_E0MvichEn3KWQ","order":27,"timestamp":1701262316}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_size_option() -> (Value, Value) {
        let query_body = r#"{"size":2,"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}}]}}}"#;

        let response = r#"{"took":6,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":10000,"relation":"gte"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"8d08b40394c2af86e88c781e6be94c197afffcd8657c9ee1edd0093891e2c353-1-5","_score":0.0,"_source":{"logAddress":"erd1l7f7qeppj39famz0d4s6dlzzqk8v53hjrprqd5v2ykkprfxny26sxy0rqk","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d61323863353900000008135f726e04c2fe000000000b555344432d333530633465000000040276a0780000000704f59e5169aac00000000a03163fcb092f45061ecf0000000564c7375403000000000010208d00000000000001b800000000655940f6","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01b8"],"shardID":1,"txOrder":0,"txHash":"8d08b40394c2af86e88c781e6be94c197afffcd8657c9ee1edd0093891e2c353","uuid":"U_Jhs8Htz7cRudrSk2RREg","order":5,"timestamp":1700348150}},{"_index":"events-000001","_id":"08c1276fdfa3cb1f91813252cdfacda3ed83f5905de6cebcae9e420b598d82db-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"a322a8114c812a57330417558b9822b4a3c287d4a22ec3888136fe82df93ea9f0000000c5745474c442d613238633539000000080de0b6b3a76400000000000b555344432d3335306334650000000401c2e8f200000007038d7ea4c680000000000a0316f7f85eb32d5fc3d70000000564af5001d200000000000fd2af00000000000001b00000000065576dc2","topics":["73776170","5745474c442d613238633539","555344432d333530633465","a322a8114c812a57330417558b9822b4a3c287d4a22ec3888136fe82df93ea9f","01b0"],"shardID":1,"txOrder":0,"txHash":"08c1276fdfa3cb1f91813252cdfacda3ed83f5905de6cebcae9e420b598d82db","uuid":"1M1MCZJiJHmw6Iz7YGlnvA","order":4,"timestamp":1700228546}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_gte_timestamp_option() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}},{"range":{"timestamp":{"gte":"1744980710"}}}]}}}"#;

        let response = r#"{"took":7,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":3,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"Tav31aOIT8aP9-LPocQsAQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472"],"txHash":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059","order":11,"timestamp":1744987250}},{"_index":"events-000001","_id":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedOutput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec"],"txOrder":0,"uuid":"4DJqRYNYSMWC5fwmV3tLVQ==","txHash":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4","order":11,"timestamp":1744986860}},{"_index":"events-000001","_id":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9-1-9","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"2kao-vyXTMyOFXRKXiL2sQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c"],"txHash":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9","order":9,"timestamp":1744984076}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_lte_timestamp_option() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}},{"range":{"timestamp":{"lte":"1744980710"}}}]}}}"#;

        let response = r#"{"took":19,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":10000,"relation":"gte"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"8d08b40394c2af86e88c781e6be94c197afffcd8657c9ee1edd0093891e2c353-1-5","_score":0.0,"_source":{"logAddress":"erd1l7f7qeppj39famz0d4s6dlzzqk8v53hjrprqd5v2ykkprfxny26sxy0rqk","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d61323863353900000008135f726e04c2fe000000000b555344432d333530633465000000040276a0780000000704f59e5169aac00000000a03163fcb092f45061ecf0000000564c7375403000000000010208d00000000000001b800000000655940f6","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01b8"],"shardID":1,"txOrder":0,"txHash":"8d08b40394c2af86e88c781e6be94c197afffcd8657c9ee1edd0093891e2c353","uuid":"U_Jhs8Htz7cRudrSk2RREg","order":5,"timestamp":1700348150}},{"_index":"events-000001","_id":"08c1276fdfa3cb1f91813252cdfacda3ed83f5905de6cebcae9e420b598d82db-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"a322a8114c812a57330417558b9822b4a3c287d4a22ec3888136fe82df93ea9f0000000c5745474c442d613238633539000000080de0b6b3a76400000000000b555344432d3335306334650000000401c2e8f200000007038d7ea4c680000000000a0316f7f85eb32d5fc3d70000000564af5001d200000000000fd2af00000000000001b00000000065576dc2","topics":["73776170","5745474c442d613238633539","555344432d333530633465","a322a8114c812a57330417558b9822b4a3c287d4a22ec3888136fe82df93ea9f","01b0"],"shardID":1,"txOrder":0,"txHash":"08c1276fdfa3cb1f91813252cdfacda3ed83f5905de6cebcae9e420b598d82db","uuid":"1M1MCZJiJHmw6Iz7YGlnvA","order":4,"timestamp":1700228546}},{"_index":"events-000001","_id":"2499d027cc66c1b14a78a2abc0c29d282c5309298347aed68c0484966ba9cdda-1-8","_score":0.0,"_source":{"originalTxHash":"43fbc29510c46ccd779063bb91f2f15ea7000bff377777624fe2b287cddf062d","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000091576d00774744da5200000000b555344432d3335306334650000000502cb85edbe00000008057eac0a19b75fa70000000a0317bc19fe36d1cf963000000005649b9081d80000000000126e2600000000000001f700000000656714c6","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"2499d027cc66c1b14a78a2abc0c29d282c5309298347aed68c0484966ba9cdda","uuid":"fKNQBSJ3l6uLdRxKIeShzg","order":8,"timestamp":1701254342}},{"_index":"events-000001","_id":"4d4351c3446b871485389ad09fd1aba38aea108dcccdda64b19002822c8dfc74-1-5","_score":0.0,"_source":{"originalTxHash":"b92300a77becad553eacf6d1edfecbbbef8d3e3eb7a7a0bbb1c4048b37d91206","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000b555344432d3335306334650000000502cb854d3e0000000c5745474c442d6132386335390000000915569d37b474aca52200000003b72c5d0000000567665ea2b90000000a0302657cc6825d22f10e0000000000126e3900000000000001f70000000065671538","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"4d4351c3446b871485389ad09fd1aba38aea108dcccdda64b19002822c8dfc74","uuid":"a6hOW6GSerN-p8i6_rVxSg","order":5,"timestamp":1701254456}},{"_index":"events-000001","_id":"8d7e6894f32273bac9be662684c227ed2c5a9d995307610ea10e6f0d0e77f4f3-1-8","_score":0.0,"_source":{"originalTxHash":"85e9d026f21a2866487d89e5cc6fde28798b5ec0e737978ad9d50b59a496630b","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000093a737cf4ab84f703330000000b555344432d3335306334650000000507458271f9000000080ef6a7281b8664190000000a033cca031405c6939028000000056020dc30c00000000000126e9f00000000000001f7000000006567179c","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"8d7e6894f32273bac9be662684c227ed2c5a9d995307610ea10e6f0d0e77f4f3","uuid":"n38RlkgkE_Cr73NISJmOSA","order":8,"timestamp":1701255068}},{"_index":"events-000001","_id":"ba62baa239eed1795556891681f9fb0eee79d676868af297c94a2246eef2c47e-1-5","_score":0.0,"_source":{"originalTxHash":"c963399f91bfe82d15a51d5b385c4ddbe4dac4171fd1ef1c936ee83ba2f66362","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000b555344432d33353063346500000005074393faf60000000c5745474c442d613238633539000000093a10b8e8a9b8b558c00000000401dc0d4e000000056762941e680000000a0302b94a2b5c0dde37680000000000126eaf00000000000001f700000000656717fc","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"ba62baa239eed1795556891681f9fb0eee79d676868af297c94a2246eef2c47e","uuid":"VI7k8Md_ORN6yF-PcQ-FHw","order":5,"timestamp":1701255164}},{"_index":"events-000001","_id":"ee638ffc03c26a1a9fd0d81d30f9a3348f1433a4d9864cc02340fd11f448c239-1-8","_score":0.0,"_source":{"originalTxHash":"89be3e76c48b1924bcb5422020a293c124c877cb12f912501255a18f5b2c651e","logAddress":"erd1qqqqqqqqqqqqqpgqh96hhj42huhe47j3jerlec7ndhw75gy72gesy7w2d6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000091308e7a5bf4f3402c40000000b555344432d3335306334650000000501c00eb2ec0000000804df76b1996c570a0000000a03ab3da41c4ac46ece7f0000000554de3da8310000000000126f8700000000000001f70000000065671d0c","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"ee638ffc03c26a1a9fd0d81d30f9a3348f1433a4d9864cc02340fd11f448c239","uuid":"1SjN0OgAKBWslxfY0JyjNw","order":8,"timestamp":1701256460}},{"_index":"events-000001","_id":"f051300339b955f01cbb9376dd3912cb5b4e1b7154c01c71f1d6bab838864306-1-5","_score":0.0,"_source":{"logAddress":"erd1f0dwxpl3vxe936cla2mkky7nym4g3xn4vgfz497dpupqul8uktzshxqj5l","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d613238633539000000080ddf9e81049e737f0000000b555344432d33353063346500000004013f942b00000007038d36e9bb8cea0000000a03abd24bb2fc19cc47670000000554d0d805b9000000000012710800000000000001f70000000065672612","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","01f7"],"shardID":1,"txOrder":0,"txHash":"f051300339b955f01cbb9376dd3912cb5b4e1b7154c01c71f1d6bab838864306","uuid":"QJwqgkwyOdOsG8DZtBEo1g","order":5,"timestamp":1701258770}},{"_index":"events-000001","_id":"05b8bb479cee9990e3bd7d516d086ff5b164598717057b7e57882d261de7b013-1-7","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqzshqdqcdzdl43vhy7p7q8uhc5xzu5x7zh2usyz5kg6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab90000000c5745474c442d61323863353900000008016345785d8a00000000000b555344432d333530633465000000031ff738000000065af3107a40000000000a03abd3ae9d8166dc07670000000554d0b80e8100000000001271d700000000000001f70000000065672aec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab9","01f7"],"shardID":1,"txOrder":0,"txHash":"05b8bb479cee9990e3bd7d516d086ff5b164598717057b7e57882d261de7b013","uuid":"iPeFxf4DPXWFmHxfbZqTQA","order":7,"timestamp":1701260012}},{"_index":"events-000001","_id":"c8786e105d5ff373fe6bb4adc08322e59c180a883682afcae2ed8dd717885ed8-1-27","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqarg523n0ja32xq6nr3dech7a4aq4xeyg5zvsfwl04y","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb0000000b555344432d3335306334650000000206180000000c5745474c442d6132386335390000000643520e2b68ba00000001010000000554d0ddcded0000000a03abd20d2da3f989395c000000000012735700000000000001f800000000656733ec","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb","01f8"],"shardID":1,"txOrder":0,"txHash":"c8786e105d5ff373fe6bb4adc08322e59c180a883682afcae2ed8dd717885ed8","uuid":"9pg7ZIVN_E0MvichEn3KWQ","order":27,"timestamp":1701262316}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_between_timestamp_option() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}},{"range":{"timestamp":{"gte":"1744986859","lte":"1744986861"}}}]}}}"#;

        let response = r#"{"took":13,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":1,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedOutput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec"],"txOrder":0,"uuid":"4DJqRYNYSMWC5fwmV3tLVQ==","txHash":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4","order":11,"timestamp":1744986860}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_sort_timestamp_ascending_option() -> (Value, Value) {
        let query_body = r#"{"sort":[{"timestamp":"asc"}],"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}}]}}}"#;

        let response = r#"{"took":16,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":10000,"relation":"gte"},"max_score":null,"hits":[{"_index":"events-000001","_id":"9cb1da74fc928afe7b7404b450e4db48e81c8d2f6c0cf35f1296357b7b221224-1-5","_score":null,"_source":{"logAddress":"erd1gadv9qg9ujkuq6em4f3ynp5ek9umj9javlewnzxm98u89um5zdksd26qsm","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e52330000000c5745474c442d6132386335390000000901158e460913d000000000000b555344432d333530633465000000041e567ff700000007470de4df8200000000000a0351c46a03bc347cbd73000000055d0b230ded00000000000c79c800000000000001540000000065434e74","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500b9757bcaaabf2f9afa519647fce3d36dddea209e5233","0154"],"shardID":1,"txOrder":0,"txHash":"9cb1da74fc928afe7b7404b450e4db48e81c8d2f6c0cf35f1296357b7b221224","uuid":"y_naBBsYGl3utYP8skW-sA","order":5,"timestamp":1698909812},"sort":[1698909812000]},{"_index":"events-000001","_id":"63c6459359ad0da8bd1851ea42f7fe01f30855c0682da5625e1ac3146c5909ef-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"7bcc2edc4a705f2340a51a2743d589d8467742e334f5b689de49f5122035a0180000000c5745474c442d6132386335390000000844004c09e76a00000000000b555344432d33353063346500000004076bc66c00000007116886276640000000000a03520858e73ff4807d73000000055d03b7478100000000000c7a1500000000000001540000000065435042","topics":["73776170","5745474c442d613238633539","555344432d333530633465","7bcc2edc4a705f2340a51a2743d589d8467742e334f5b689de49f5122035a018","0154"],"shardID":1,"txOrder":0,"txHash":"63c6459359ad0da8bd1851ea42f7fe01f30855c0682da5625e1ac3146c5909ef","uuid":"GQqrqksFmZa2W0b0E9w4lQ","order":4,"timestamp":1698910274},"sort":[1698910274000]},{"_index":"events-000001","_id":"af474e27744191dd1943c53a44931ef4362ec95a2f9b705e9bca1c46396d7571-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"0c916ef41ea31247ecd4064a661667ca6a2822326aab0a1638fa0e5d0bbcade90000000c5745474c442d613238633539000000093635c9adc5dea000000000000b555344432d33353063346500000005058f8178e7000000080de0b6b3a76400000000000a03883041de522bbc7d7300000005577435ce9a00000000000c7d8a00000000000001550000000065436500","topics":["73776170","5745474c442d613238633539","555344432d333530633465","0c916ef41ea31247ecd4064a661667ca6a2822326aab0a1638fa0e5d0bbcade9","0155"],"shardID":1,"txOrder":0,"txHash":"af474e27744191dd1943c53a44931ef4362ec95a2f9b705e9bca1c46396d7571","uuid":"gsYVEef5eVQHCb63fSEhxQ","order":4,"timestamp":1698915584},"sort":[1698915584000]},{"_index":"events-000001","_id":"2ee146d04a0efb004202ec9b05d3aeebf3e17958e5f8ad34ac596c89ee77fc5f-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"8aa90bcd75175da4cc6ebf7e91c64d209f7fe223f5d17ca680b699ef85b712970000000c5745474c442d61323863353900000009015af1d78b58c400000000000b555344432d333530633465000000042167fdfe0000000758d15e176280000000000a03898adae47f6d1dfd73000000055752cdd09c00000000000c825b000000000000015500000000654381e6","topics":["73776170","5745474c442d613238633539","555344432d333530633465","8aa90bcd75175da4cc6ebf7e91c64d209f7fe223f5d17ca680b699ef85b71297","0155"],"shardID":1,"txOrder":0,"txHash":"2ee146d04a0efb004202ec9b05d3aeebf3e17958e5f8ad34ac596c89ee77fc5f","uuid":"E_kY-rKdZXXaAM8cZ9rYDg","order":4,"timestamp":1698922982},"sort":[1698922982000]},{"_index":"events-000001","_id":"ec36bd50f274587b01728230dd0ad41c14b9f0bde971d5fe9f5711070650af2e-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"1ba007a4d21e3252d297633f100c3f2e8dbe70d8b977a77ee6368507cd4c8dd80000000c5745474c442d613238633539000000080de0b6b3a76400000000000b555344432d3335306334650000000401558c3f00000007038d7ea4c680000000000a038998b80db46fbb7d7300000005575178445d00000000000c846100000000000001550000000065438e0a","topics":["73776170","5745474c442d613238633539","555344432d333530633465","1ba007a4d21e3252d297633f100c3f2e8dbe70d8b977a77ee6368507cd4c8dd8","0155"],"shardID":1,"txOrder":0,"txHash":"ec36bd50f274587b01728230dd0ad41c14b9f0bde971d5fe9f5711070650af2e","uuid":"Aw3vmbg9yJRI5sHRAMKuzQ","order":4,"timestamp":1698926090},"sort":[1698926090000]},{"_index":"events-000001","_id":"816a2332ac50c87a332a65595c3c9c6b0045124984929461256f684b5c021f43-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"2ff39484c3f076afb43efeacc9821b2d7a48b36e5a7c10784279dc1c8323dad90000000c5745474c442d6132386335390000000829a2241af62c00000000000b555344432d333530633465000000040400661b000000070aa87bee5380000000000a0389c24f89537793fd7300000005574d77de4200000000000c887c0000000000000156000000006543a6ac","topics":["73776170","5745474c442d613238633539","555344432d333530633465","2ff39484c3f076afb43efeacc9821b2d7a48b36e5a7c10784279dc1c8323dad9","0156"],"shardID":1,"txOrder":0,"txHash":"816a2332ac50c87a332a65595c3c9c6b0045124984929461256f684b5c021f43","uuid":"rfrJd3-t0UM_NQsAc0iHqw","order":4,"timestamp":1698932396},"sort":[1698932396000]},{"_index":"events-000001","_id":"0b38d4adc2aefe5a18a2e80c250ae4c750e24ece23e89ef197acaf40ee65e940-1-8","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq5ycddehv6076nzmf3ccynhu4tlrc3068kklszj95gn","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf0000000b555344432d333530633465000000033d09000000000c5745474c442d61323863353900000008027754562a02bb6b000000020fa000000005574db4d7a20000000a0389bfd834fd4d91420800000000000c88bb0000000000000156000000006543a826","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf","0156"],"shardID":1,"txOrder":0,"txHash":"0b38d4adc2aefe5a18a2e80c250ae4c750e24ece23e89ef197acaf40ee65e940","uuid":"XBQGc9jY26ILMYnsuAUn8Q","order":8,"timestamp":1698932774},"sort":[1698932774000]},{"_index":"events-000001","_id":"e230d2f752499e96ffdeb9bac687a0bbe135735ea1aec094b0e43498c1dfc2af-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq5ycddehv6076nzmf3ccynhu4tlrc3068kklszj95gn","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf0000000c5745474c442d61323863353900000008027754562a02bb6b0000000b555344432d333530633465000000033cab6400000006a19ece60c54e0000000a0389c24ee7b4a933382500000005574d782c3e00000000000c88cb0000000000000156000000006543a886","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf","0156"],"shardID":1,"txOrder":0,"txHash":"e230d2f752499e96ffdeb9bac687a0bbe135735ea1aec094b0e43498c1dfc2af","uuid":"SEyaZ3Mj3abLIPp1yoiG8A","order":4,"timestamp":1698932870},"sort":[1698932870000]},{"_index":"events-000001","_id":"acf8efd370364abb9b6fa5a5d5499775b65a0515aa2a04470e9a2d15632807ff-1-8","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq5ycddehv6076nzmf3ccynhu4tlrc3068kklszj95gn","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf0000000b555344432d333530633465000000033d09000000000c5745474c442d6132386335390000000802775453856b86f7000000020fa000000005574db5259e0000000a0389bfd7936123c7b12e00000000000c88d70000000000000156000000006543a8ce","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf","0156"],"shardID":1,"txOrder":0,"txHash":"acf8efd370364abb9b6fa5a5d5499775b65a0515aa2a04470e9a2d15632807ff","uuid":"K5ygL_jRLqg616jh0nMflw","order":8,"timestamp":1698932942},"sort":[1698932942000]},{"_index":"events-000001","_id":"1745c367c5d1b6b194c12cc01046eb27bb33d943eb60e936bfbc52ecb4a8a385-1-4","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq5ycddehv6076nzmf3ccynhu4tlrc3068kklszj95gn","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf0000000c5745474c442d6132386335390000000802775453856b86f70000000b555344432d333530633465000000033cab6400000006a19ecdb390430000000a0389c24e4615db7fa7e200000005574d787a3a00000000000c88da0000000000000156000000006543a8e0","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500a130d6e6ecd3fda98b698e3049df955fc788bf47b5bf","0156"],"shardID":1,"txOrder":0,"txHash":"1745c367c5d1b6b194c12cc01046eb27bb33d943eb60e936bfbc52ecb4a8a385","uuid":"k26yGyCgB3OlUi95L2yPjA","order":4,"timestamp":1698932960},"sort":[1698932960000]}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_sort_timestamp_descending_option() -> (Value, Value) {
        let query_body = r#"{"sort":[{"timestamp":"desc"}],"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}}]}}}"#;

        let response = r#"{"took":121,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":10000,"relation":"gte"},"max_score":null,"hits":[{"_index":"events-000001","_id":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059-1-11","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"Tav31aOIT8aP9-LPocQsAQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472"],"txHash":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059","order":11,"timestamp":1744987250},"sort":[1744987250000]},{"_index":"events-000001","_id":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4-1-11","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedOutput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec"],"txOrder":0,"uuid":"4DJqRYNYSMWC5fwmV3tLVQ==","txHash":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4","order":11,"timestamp":1744986860},"sort":[1744986860000]},{"_index":"events-000001","_id":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9-1-9","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"2kao-vyXTMyOFXRKXiL2sQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c"],"txHash":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9","order":9,"timestamp":1744984076},"sort":[1744984076000]},{"_index":"events-000001","_id":"89d5ce6f8a6b1909bf2c599a24c8f5db6a1302753f9c928bab9763b1b38d4575-1-11","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqzshqdqcdzdl43vhy7p7q8uhc5xzu5x7zh2usyz5kg6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab90000000b555344432d33353063346500000003b498100000000c5745474c442d6132386335390000000808c69be0bdf49cb8000000022e3b000000057da59da0350000000a061fbafe630a1f1ed55200000000008024a90000000000000dae0000000067ffb782","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab9","0dae"],"shardID":1,"additionalData":["00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab90000000b555344432d33353063346500000003b498100000000c5745474c442d6132386335390000000808c69be0bdf49cb8000000022e3b000000057da59da0350000000a061fbafe630a1f1ed55200000000008024a90000000000000dae0000000067ffb782"],"txOrder":0,"uuid":"9FZu1FAAQx-5IRl4HLba7g==","txHash":"89d5ce6f8a6b1909bf2c599a24c8f5db6a1302753f9c928bab9763b1b38d4575","order":11,"timestamp":1744811906},"sort":[1744811906000]},{"_index":"events-000001","_id":"0bc4e438f7e25a79bf0e046a87b06ff7796dd5f471a343d986423afde4a479b3-1-11","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqzshqdqcdzdl43vhy7p7q8uhc5xzu5x7zh2usyz5kg6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab90000000b555344432d3335306334650000000403158f490000000c5745474c442d61323863353900000008265f5d7ee44c7a6200000002ca20000000057da4e91f420000000a061fc3c61f5b5575193e000000000080245b0000000000000dae0000000067ffb5ae","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab9","0dae"],"shardID":1,"additionalData":["00000000000000000500142e06830d137f58b2e4f07c03f2f8a185ca1bc2bab90000000b555344432d3335306334650000000403158f490000000c5745474c442d61323863353900000008265f5d7ee44c7a6200000002ca20000000057da4e91f420000000a061fc3c61f5b5575193e000000000080245b0000000000000dae0000000067ffb5ae"],"txOrder":0,"uuid":"Bn19nPtGTCGVbbPE0Y-yfQ==","txHash":"0bc4e438f7e25a79bf0e046a87b06ff7796dd5f471a343d986423afde4a479b3","order":11,"timestamp":1744811438},"sort":[1744811438000]},{"_index":"events-000001","_id":"b513423931cfd5214b8e801bab2cd938b0c66f235f9a398c568a7ccc789a03be-1-7","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"75cb87c24351a67b892f57dcec0eb2b2a07aafab2f1aab741a10fc61059f2fe80000000c5745474c442d6132386335390000000806f05b59d3b200000000000b555344432d333530633465000000038de8cf0000000701c6bf526340000000000a061fea2a69e43f0b81ba000000057da1d3f50900000000008020300000000000000dae0000000067ff9cac","topics":["73776170","5745474c442d613238633539","555344432d333530633465","75cb87c24351a67b892f57dcec0eb2b2a07aafab2f1aab741a10fc61059f2fe8","0dae"],"shardID":1,"txOrder":0,"uuid":"Bhv1XkJaRhS7eBRLfpL0fQ==","additionalData":["75cb87c24351a67b892f57dcec0eb2b2a07aafab2f1aab741a10fc61059f2fe80000000c5745474c442d6132386335390000000806f05b59d3b200000000000b555344432d333530633465000000038de8cf0000000701c6bf526340000000000a061fea2a69e43f0b81ba000000057da1d3f50900000000008020300000000000000dae0000000067ff9cac"],"txHash":"b513423931cfd5214b8e801bab2cd938b0c66f235f9a398c568a7ccc789a03be","order":7,"timestamp":1744805036},"sort":[1744805036000]},{"_index":"events-000001","_id":"6339a3a07d18839e9865e2bf24e6fa7a72a26bfffeac9f911d8785da1bd4c4dc-1-14","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqa7hv0nahgsl8tz0psat46x0tchm0wuyc0n4s6q28ad","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb0000000b555344432d33353063346500000004123071e90000000c5745474c442d61323863353900000008e271d3b4287fdb2a0000000304a80c000000057da261ddd80000000a061fe33bd549bdbcc1ba0000000000801a4b0000000000000dad0000000067ff7948","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb","0dad"],"shardID":1,"txOrder":0,"uuid":"J0AsJNuMSHSZapR83F7-Ug==","additionalData":["00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb0000000b555344432d33353063346500000004123071e90000000c5745474c442d61323863353900000008e271d3b4287fdb2a0000000304a80c000000057da261ddd80000000a061fe33bd549bdbcc1ba0000000000801a4b0000000000000dad0000000067ff7948"],"txHash":"6339a3a07d18839e9865e2bf24e6fa7a72a26bfffeac9f911d8785da1bd4c4dc","order":14,"timestamp":1744795976},"sort":[1744795976000]},{"_index":"events-000001","_id":"b12c96b226660650793bae9e3847716a6026fc5344ad945d5d73ea4342f4e610-1-7","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa894020000000b555344432d3335306334650000000227100000000c5745474c442d6132386335390000000701e693fb67120b000000010a000000057d9033bff50000000a0620c5cab73b2dc0bbb900000000008016a70000000000000dad0000000067ff634c","topics":["73776170","555344432d333530633465","5745474c442d613238633539","2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa89402","0dad"],"shardID":1,"txOrder":0,"uuid":"bmnMAmWNSuWNsSuxZSUvlw==","additionalData":["2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa894020000000b555344432d3335306334650000000227100000000c5745474c442d6132386335390000000701e693fb67120b000000010a000000057d9033bff50000000a0620c5cab73b2dc0bbb900000000008016a70000000000000dad0000000067ff634c"],"txHash":"b12c96b226660650793bae9e3847716a6026fc5344ad945d5d73ea4342f4e610","order":7,"timestamp":1744790348},"sort":[1744790348000]},{"_index":"events-000001","_id":"5bd9a4146ec375829774be784f179fabb6f96e5f695a63acd176b757ccb577dd-1-7","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa894020000000b555344432d3335306334650000000227100000000c5745474c442d6132386335390000000701e693fc955d06000000010a000000057d903398ea0000000a0620c5cc9e0da152466b00000000008016a20000000000000dad0000000067ff632e","topics":["73776170","555344432d333530633465","5745474c442d613238633539","2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa89402","0dad"],"shardID":1,"txOrder":0,"uuid":"Ifn2NhyWTrW6peiUC9k2Fg==","additionalData":["2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa894020000000b555344432d3335306334650000000227100000000c5745474c442d6132386335390000000701e693fc955d06000000010a000000057d903398ea0000000a0620c5cc9e0da152466b00000000008016a20000000000000dad0000000067ff632e"],"txHash":"5bd9a4146ec375829774be784f179fabb6f96e5f695a63acd176b757ccb577dd","order":7,"timestamp":1744790318},"sort":[1744790318000]},{"_index":"events-000001","_id":"07446656d0b9f024c9e2dc7c162edc20059e989c8756e5e90524fb0c8200bda9-1-7","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa894020000000b555344432d3335306334650000000227100000000c5745474c442d6132386335390000000701e693fdc3a803000000010a000000057d903371df0000000a0620c5ce84e0161242e700000000008016920000000000000dac0000000067ff62ce","topics":["73776170","555344432d333530633465","5745474c442d613238633539","2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa89402","0dac"],"shardID":1,"txOrder":0,"uuid":"hovy0yFOQvyj7asdH1TLFQ==","additionalData":["2aaaed9202f29a0ea5b408f4a1e9d2398a0af2dc34370845e7569aa58aa894020000000b555344432d3335306334650000000227100000000c5745474c442d6132386335390000000701e693fdc3a803000000010a000000057d903371df0000000a0620c5ce84e0161242e700000000008016920000000000000dac0000000067ff62ce"],"txHash":"07446656d0b9f024c9e2dc7c162edc20059e989c8756e5e90524fb0c8200bda9","order":7,"timestamp":1744790222},"sort":[1744790222000]}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_with_one_field_filter_no_options() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}},{"term":{"topics":"0dba"}}]}}}"#;

        let response = r#"{"took":41,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":3,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"Tav31aOIT8aP9-LPocQsAQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472"],"txHash":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059","order":11,"timestamp":1744987250}},{"_index":"events-000001","_id":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9-1-9","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"2kao-vyXTMyOFXRKXiL2sQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c"],"txHash":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9","order":9,"timestamp":1744984076}},{"_index":"events-000001","_id":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedOutput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec"],"txOrder":0,"uuid":"4DJqRYNYSMWC5fwmV3tLVQ==","txHash":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4","order":11,"timestamp":1744986860}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_with_two_fields_filter_no_options() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}},{"term":{"topics":"5745474c442d613238633539"}},{"term":{"topics":"0dba"}}]}}}"#;

        let response = r#"{"took":55,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":3,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"Tav31aOIT8aP9-LPocQsAQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d613238633539000000098eb75edd2af06d40000000000b555344432d333530633465000000050b3548a2760000000824890e0b901cc8000000000a0676843a803aeac36da4000000057718ca39ba00000000008096d10000000000000dba0000000068026472"],"txHash":"27e2f43d2f062635ee06ead710f26a6d1f3e2bc6c092d102c48ef68f9e2b4059","order":11,"timestamp":1744987250}},{"_index":"events-000001","_id":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9-1-9","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"txOrder":0,"uuid":"2kao-vyXTMyOFXRKXiL2sQ==","additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000b555344432d3335306334650000000504a817c8000000000c5745474c442d6132386335390000000937dd0b0bb801e5b5a70000000401312d0000000005824eab72b00000000a05e7ea76ad8a12c3f97700000000008094c00000000000000dba000000006802580c"],"txHash":"a9d6c45f55fdc28e0b3ff966e0062ac92c45315febfee030796024c3377ea0a9","order":9,"timestamp":1744984076}},{"_index":"events-000001","_id":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4-1-11","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedOutput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec"],"txOrder":0,"uuid":"4DJqRYNYSMWC5fwmV3tLVQ==","txHash":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4","order":11,"timestamp":1744986860}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_with_all_fields_filter_and_all_options() -> (Value, Value) {
        let query_body = r#"{"from":0,"size":10,"sort":[{"timestamp":"asc"}],"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}},{"term":{"topics":"5745474c442d613238633539"}},{"term":{"topics":"555344432d333530633465"}},{"term":{"topics":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb"}},{"term":{"topics":"0dba"}},{"range":{"timestamp":{"gte":"1744986859","lte":"1744986861"}}}]}}}"#;

        let response = r#"{"took":23,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":1,"relation":"eq"},"max_score":null,"hits":[{"_index":"events-000001","_id":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4-1-11","_score":null,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe","identifier":"swapTokensFixedOutput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec","topics":["73776170","5745474c442d613238633539","555344432d333530633465","00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb","0dba"],"shardID":1,"additionalData":["00000000000000000500d39058120e411e5b4c6e52d1a0781972a67149c67ceb0000000c5745474c442d6132386335390000000806efca2b9f85e7280000000b555344432d333530633465000000039896800000000701c69a27d6eafb0000000a05e7f164b11b8a72f5a400000005824e12dc3000000000008096900000000000000dba00000000680262ec"],"txOrder":0,"uuid":"4DJqRYNYSMWC5fwmV3tLVQ==","txHash":"6947b73cdc82c67eab1526d36274a5606edb98b66973aa76921f0630b06c78b4","order":11,"timestamp":1744986860},"sort":[1744986860000]}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_swaps_no_filter_with_from_option() -> (Value, Value) {
        let query_body = r#"{"from":10,"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"}},{"term":{"topics":"73776170"}}]}}}"#;

        let response = r#"{"took":38,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":10000,"relation":"gte"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"7d435face2335291b882fe4c587a37fca143226719eaf197177770ed45c9df5a-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"0de8a9b8eb04c88ae1980e46273b81512260c90c5664415ed4e13fefdd9c14eb0000000c5745474c442d6132386335390000000806f05b59d3b200000000000b555344432d333530633465000000039f46490000000701c6bf526340000000000a03ad78e85cbfb15db15c0000000554aad919ee000000000012759600000000000001f80000000065674166","topics":["73776170","5745474c442d613238633539","555344432d333530633465","0de8a9b8eb04c88ae1980e46273b81512260c90c5664415ed4e13fefdd9c14eb","01f8"],"shardID":1,"txOrder":0,"txHash":"7d435face2335291b882fe4c587a37fca143226719eaf197177770ed45c9df5a","uuid":"AR70Xo9RRDTklYoYtMyEKw","order":4,"timestamp":1701265766}},{"_index":"events-000001","_id":"35bac2e23d23773d52bdae8e247a810fed4d001c1ecf02fffd51447ce10956d8-1-7","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqg5uax0mmx36q68z2ksuh9ehzsf36d7jfvr2shnsrck","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"000000000000000005004539d33f7b34740d1c4ab43972e6e28263a6fa4960d50000000c5745474c442d613238633539000000080de0b6b3a76400000000000b555344432d33353063346500000004013e929300000007038d7ea4c680000000000a03ad73913eaf72dc56330000000554ab56e9da0000000000127c1400000000000001f9000000006567685a","topics":["73776170","5745474c442d613238633539","555344432d333530633465","000000000000000005004539d33f7b34740d1c4ab43972e6e28263a6fa4960d5","01f9"],"shardID":1,"txOrder":0,"txHash":"35bac2e23d23773d52bdae8e247a810fed4d001c1ecf02fffd51447ce10956d8","uuid":"STZ9RmBdTsrGdQ2LnJ8FWA","order":7,"timestamp":1701275738}},{"_index":"events-000001","_id":"8316262588698cb63341707d27e0b0ad1b8c7c712a609223d743c9d32c0fdb14-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779c43d7033f70b0000000203e80000000563c06541a70000000a031e6b88aadfd5290660000000000012430700000000000001f20000000065661200","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f2"],"shardID":1,"txOrder":0,"txHash":"8316262588698cb63341707d27e0b0ad1b8c7c712a609223d743c9d32c0fdb14","uuid":"q6uQp0l9WF0dHLB9MY5IXw","order":4,"timestamp":1701188096}},{"_index":"events-000001","_id":"da206fd650d910877ec4cbd1b2b9be635e0289e2796daf5c1744424947979d19-1-6","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq5wmxdypdta9m7rlexay0hy26adp3yn9lv5ys7xpyez","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb0000000b555344432d33353063346500000001130000000c5745474c442d61323863353900000005979f157f52000000000000000563c06541ba0000000a031e6b88aa483613870e000000000012430a00000000000001f20000000065661212","topics":["73776170","555344432d333530633465","5745474c442d613238633539","00000000000000000500efaec7cfb7443e7589e187575d19ebc5f6f770987ceb","01f2"],"shardID":1,"txOrder":0,"txHash":"da206fd650d910877ec4cbd1b2b9be635e0289e2796daf5c1744424947979d19","uuid":"cicxcEPE60v5h0JeuvhoBg","order":6,"timestamp":1701188114}},{"_index":"events-000001","_id":"d67c96176b257fba0daa9d0435116d74fa735a2f0d85f6e70ac5d8b04731a51d-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779b67e0cb6d2260000000203e80000000563c608b94f0000000a031e3e80b3ae59609474000000000012443300000000000001f30000000065661908","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f3"],"shardID":1,"txOrder":0,"txHash":"d67c96176b257fba0daa9d0435116d74fa735a2f0d85f6e70ac5d8b04731a51d","uuid":"Pd-BUNNw4tE45zu15fyadA","order":4,"timestamp":1701189896}},{"_index":"events-000001","_id":"1057c40980d41f1b025380d788b334f3508720be5ea54ed60447ed5987386332-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779b52fad4819720000000203e80000000563c691ea670000000a031e3a394fc7c819d367000000000012454200000000000001f30000000065661f62","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f3"],"shardID":1,"txOrder":0,"txHash":"1057c40980d41f1b025380d788b334f3508720be5ea54ed60447ed5987386332","uuid":"3sKPcf8wCF4NHk-SeyM5uQ","order":4,"timestamp":1701191522}},{"_index":"events-000001","_id":"3f3f4ebccbcdd791a855660d520f2e9f9d0ae27ecd5e3507d0583fd80ab2445f-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779b49b12d152ad0000000203e80000000563c6cee3c70000000a031e38527c7c9577731f000000000012458400000000000001f300000000656620ee","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f3"],"shardID":1,"txOrder":0,"txHash":"3f3f4ebccbcdd791a855660d520f2e9f9d0ae27ecd5e3507d0583fd80ab2445f","uuid":"Q5smAiKjzDeGHUblHCg95Q","order":4,"timestamp":1701191918}},{"_index":"events-000001","_id":"79ef99b338a648e098e3ac9c12bffcb711004a1b23bad23365685641a6be0a4b-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779b475ec5e2b2c0000000203e80000000563c6de221f0000000a031e37d8c806a91947f3000000000012458500000000000001f300000000656620f4","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f3"],"shardID":1,"txOrder":0,"txHash":"79ef99b338a648e098e3ac9c12bffcb711004a1b23bad23365685641a6be0a4b","uuid":"S7qFvXIJAhkdRI6hNIoK3w","order":4,"timestamp":1701191924}},{"_index":"events-000001","_id":"b472a6b2f3d8e8141c32e7bc4793a39cceec294c9bcfa8442feb3fa51293d363-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779b406796acc750000000203e80000000563c70bdd270000000a031e366bab83ca078bba00000000001245a600000000000001f300000000656621ba","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f3"],"shardID":1,"txOrder":0,"txHash":"b472a6b2f3d8e8141c32e7bc4793a39cceec294c9bcfa8442feb3fa51293d363","uuid":"VSw1eK6Y_iW9cShp2z2HNQ","order":4,"timestamp":1701192122}},{"_index":"events-000001","_id":"8e452a3114a86235e86856c773e27dfef871490a70d081bd7d2e6ef5ed346cbb-1-4","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b0000000b555344432d333530633465000000030f42400000000c5745474c442d6132386335390000000779b3bc2d1da0bf0000000203e80000000563c72a59d70000000a031e357843e649ae364b000000000012463e00000000000001f3000000006566254a","topics":["73776170","555344432d333530633465","5745474c442d613238633539","62ca3f818cf8cabb1a82ae70fcb51ca6d76cdd1e012a527e9bb1974164f6dc4b","01f3"],"shardID":1,"txOrder":0,"txHash":"8e452a3114a86235e86856c773e27dfef871490a70d081bd7d2e6ef5ed346cbb","uuid":"vNMMn2Sas-STmP0lFgN6KA","order":4,"timestamp":1701193034}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_empty_events_no_filter_no_options() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c"}},{"term":{"topics":"656d7074794576656e74"}}]}}}"#;

        let response = r#"{"took":28,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":2,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"529f68b2e497d1b01872a98efd4469d1b429989056fc205cdd010011a2e7a5bd-1-0","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","identifier":"emitEmptyEvent","address":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","topics":["656d7074794576656e74"],"shardID":1,"additionalData":[""],"txOrder":0,"uuid":"rGjCHkh5QFWW2K1VTTkiMA==","txHash":"529f68b2e497d1b01872a98efd4469d1b429989056fc205cdd010011a2e7a5bd","order":0,"timestamp":1745249390}},{"_index":"events-000001","_id":"4c206aa3bdda5bbd2eb35650550577fd954cffc64b391fd53085c3baaacdfb46-1-0","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","identifier":"emitEmptyEvent","address":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","topics":["656d7074794576656e74"],"shardID":1,"additionalData":[""],"txOrder":0,"uuid":"77f1yI7WSqugNEO_dziobw==","txHash":"4c206aa3bdda5bbd2eb35650550577fd954cffc64b391fd53085c3baaacdfb46","order":0,"timestamp":1745250830}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_events_with_only_data_no_filter_no_options() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c"}},{"term":{"topics":"6576656e74576974684f6e6c7944617461"}}]}}}"#;

        let response = r#"{"took":6,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":2,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"3b34262f0217745affa0bd4815f9bd7cde1d29e4998494291d4d142b60d8ef11-1-0","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","identifier":"emitEventWithOnlyData","address":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","data":"bb6fa113c74ac30ee767aea8d2fa29fdbae24e8e731f3e2930e91ba993a54449000000020230","topics":["6576656e74576974684f6e6c7944617461"],"shardID":1,"txOrder":0,"uuid":"Ws_-RRa5SROwwnpPR64lyw==","additionalData":["bb6fa113c74ac30ee767aea8d2fa29fdbae24e8e731f3e2930e91ba993a54449000000020230"],"txHash":"3b34262f0217745affa0bd4815f9bd7cde1d29e4998494291d4d142b60d8ef11","order":0,"timestamp":1745249372}},{"_index":"events-000001","_id":"5826b45a340fc68f2e39ece57f0d08c71f09220ef3894d65a66b6f6f77ad7137-1-0","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","identifier":"emitEventWithOnlyData","address":"erd1qqqqqqqqqqqqqpgq6qamu3fk4qtz8fqd8pnw8s5sgzvt4mspg3yskwxvu3","data":"bb6fa113c74ac30ee767aea8d2fa29fdbae24e8e731f3e2930e91ba993a54449000000020235","topics":["6576656e74576974684f6e6c7944617461"],"shardID":1,"additionalData":["bb6fa113c74ac30ee767aea8d2fa29fdbae24e8e731f3e2930e91ba993a54449000000020235"],"txOrder":0,"uuid":"6r-X7ywvQi2XLaW3phKYPQ==","txHash":"5826b45a340fc68f2e39ece57f0d08c71f09220ef3894d65a66b6f6f77ad7137","order":0,"timestamp":1745250848}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }

    fn query_events_with_multi_value_encoded_and_data_no_filter_no_options() -> (Value, Value) {
        let query_body = r#"{"query":{"bool":{"filter":[{"match":{"address":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c"}},{"term":{"topics":"6576656e74576974684d756c746956616c7565456e636f646564"}}]}}}"#;

        let response = r#"{"took":6,"timed_out":false,"_shards":{"total":5,"successful":5,"skipped":0,"failed":0},"hits":{"total":{"value":2,"relation":"eq"},"max_score":0.0,"hits":[{"_index":"events-000001","_id":"4bb80f37b2d2d6fc90217f4c359f40041f8c936eb5fe37f9d0bba6d8d70fc0f9-1-0","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c","identifier":"emitEventWithMultiValueEncoded","address":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c","data":"05","topics":["6576656e74576974684d756c746956616c7565456e636f646564","01","02","03","04"],"shardID":1,"additionalData":["05"],"txOrder":0,"uuid":"1U_trs0pRKydQelkUBplOA==","txHash":"4bb80f37b2d2d6fc90217f4c359f40041f8c936eb5fe37f9d0bba6d8d70fc0f9","order":0,"timestamp":1745764742}},{"_index":"events-000001","_id":"201be66b1f9e3ab7486fe1d3e090a7ee7cc25669587d7fad102688ced97dddda-1-0","_score":0.0,"_source":{"logAddress":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c","identifier":"emitEventWithMultiValueEncoded","address":"erd1qqqqqqqqqqqqqpgqd8jlfyz7sr7unxlagc0e8u2t96pyt5g6g3ysjwje0c","data":"20","topics":["6576656e74576974684d756c746956616c7565456e636f646564","02","04","08","10"],"shardID":1,"additionalData":["20"],"txOrder":0,"uuid":"7GzlQjdbSkWchhoNqDu1IQ==","txHash":"201be66b1f9e3ab7486fe1d3e090a7ee7cc25669587d7fad102688ced97dddda","order":0,"timestamp":1745764766}}]}}"#;

        (
            Value::from_str(query_body).unwrap(),
            Value::from_str(response).unwrap(),
        )
    }
}

fn get_executor() -> Arc<BaseElasticSearchNodeQueryExecutor<ElasticSearchNodeProxy<MockElasticSearchClient>>> {
    let executor = BaseElasticSearchNodeQueryExecutor::new("".to_string());
    Arc::new(executor)
}

#[tokio::test]
async fn test_query_events_no_filter_no_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            None,
            None
        )
        .await?;

    let first_element = result.get(0).unwrap();

    let expected_len = 10;
    let expected_first_result_timestamp = 1700348150;

    assert_eq!(result.len(), expected_len);
    assert_eq!(first_element.timestamp, expected_first_result_timestamp);

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_size_option_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    size: Some(2),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let first_element = result.get(0).unwrap();

    let expected_len = 2;
    let expected_first_result_timestamp = 1700348150;

    assert_eq!(result.len(), expected_len);
    assert_eq!(first_element.timestamp, expected_first_result_timestamp);

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_from_option_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    from: Some(10),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let first_element = result.get(0).unwrap();

    let expected_len = 10;
    let expected_first_result_timestamp = 1701265766;

    assert_eq!(result.len(), expected_len);
    assert_eq!(first_element.timestamp, expected_first_result_timestamp);

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_gte_timestamp_option_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let gte_timestamp = 1744980710;

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    timestamp: Some(TimestampOption::GreaterThanOrEqual(gte_timestamp)),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let expected_len = 3;
    assert_eq!(result.len(), expected_len);

    for element in result {
        assert!(element.timestamp >= gte_timestamp);
    };

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_lte_timestamp_option_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let gte_timestamp = 1744980710;

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    timestamp: Some(TimestampOption::LowerThanOrEqual(gte_timestamp)),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let expected_len = 10;
    assert_eq!(result.len(), expected_len);

    for element in result {
        assert!(element.timestamp <= gte_timestamp);
    };

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_between_timestamp_option_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    timestamp: Some(TimestampOption::Between(1744986859, 1744986861)),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let expected_len = 1;
    assert_eq!(result.len(), expected_len);

    let result = result.get(0).unwrap();

    assert_eq!(result.timestamp, 1744986860);

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_sort_timestamp_ascending_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    sort: Some(
                        QueryEventsSortOptions {
                            timestamp: Some(SortOption::Ascending)
                        }
                    ),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let expected_len = 10;
    assert_eq!(result.len(), expected_len);

    let mut result_iter = result.into_iter();
    let mut previous_timestamp = result_iter.next().unwrap().timestamp;

    for element in result_iter {
        assert!(previous_timestamp <= element.timestamp);
        previous_timestamp = element.timestamp;
    }

    Ok(())
}

#[tokio::test]
async fn test_query_events_no_filter_with_sort_timestamp_descending_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(
                EventQueryOptions {
                    sort: Some(
                        QueryEventsSortOptions {
                            timestamp: Some(SortOption::Descending)
                        }
                    ),
                    ..Default::default()
                }
            ),
            None
        )
        .await?;

    let expected_len = 10;
    assert_eq!(result.len(), expected_len);

    let mut result_iter = result.into_iter();
    let mut previous_timestamp = result_iter.next().unwrap().timestamp;

    for element in result_iter {
        assert!(previous_timestamp >= element.timestamp);
        previous_timestamp = element.timestamp;
    }

    Ok(())
}

#[tokio::test]
async fn test_query_events_with_one_field_filter_no_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let epoch = 3514;

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            None,
            Some(
                SwapEventFilterOptions {
                    epoch: Some(epoch),
                    ..Default::default()
                }
            )
        )
        .await?;

    let expected_len = 3;
    assert_eq!(result.len(), expected_len);

    for element in result {
        assert_eq!(element.event.epoch, epoch);
    };

    Ok(())
}

#[tokio::test]
async fn test_query_events_with_two_fields_filter_no_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let token_in = "WEGLD-a28c59".to_string();
    let epoch = 3514;

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            None,
            Some(
                SwapEventFilterOptions {
                    token_in: Some(token_in.clone()),
                    epoch: Some(epoch),
                    ..Default::default()
                }
            )
        )
        .await?;

    let expected_len = 2;
    assert_eq!(result.len(), expected_len);

    for element in result {
        assert_eq!(element.event.token_in, token_in);
        assert_eq!(element.event.epoch, epoch);
    };

    Ok(())
}

#[tokio::test]
async fn test_query_events_with_all_fields_filter_and_all_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = PairContract::new(
        TESTER_POOL_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .swap(
            Some(EventQueryOptions {
                from: Some(0),
                size: Some(10),
                timestamp: Some(TimestampOption::Between(1744986859, 1744986861)),
                sort: Some(
                    QueryEventsSortOptions {
                        timestamp: Some(SortOption::Ascending)
                    }
                ),
            }),
            Some(SwapEventFilterOptions {
                token_in: Some("WEGLD-a28c59".to_string()),
                token_out: Some("USDC-350c4e".to_string()),
                caller: Some(Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe").unwrap()),
                epoch: Some(3514)
            }),
        )
        .await?;

    let expected_len = 1;
    assert_eq!(result.len(), expected_len);

    let result = result.get(0).unwrap();
    let expected_result = EventQueryResult {
        timestamp: 1744986860,
        event: SwapEventQueryResult {
            token_in: "WEGLD-a28c59".to_string(),
            token_out: "USDC-350c4e".to_string(),
            caller: Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe")?,
            epoch: 3514,
            swap_event: SwapEvent {
                caller: Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq6wg9syswgy09knrw2tg6q7qew2n8zjwx0n4s377sfe")?,
                token_id_in: "WEGLD-a28c59".to_string(),
                token_amount_in: BigUint::from_str("499840372370171688").unwrap(),
                token_id_out: "USDC-350c4e".to_string(),
                token_amount_out: BigUint::from_str("10000000").unwrap(),
                fee_amount: BigUint::from_str("499840372370171").unwrap(),
                token_in_reserve: BigUint::from_str("27890424517767789213092").unwrap(),
                token_out_reserve: BigUint::from_str("559655607344").unwrap(),
                block: 8427152,
                epoch: 3514,
                timestamp: 1744986860,
            },
        },
    };

    assert_eq!(result, &expected_result);

    Ok(())
}

#[tokio::test]
async fn test_query_empty_events_no_filter_no_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .empty_event(
            None,
            None
        )
        .await?;

    let expected_len = 2;
    assert_eq!(result.len(), expected_len);

    assert_eq!(
        result,
        vec![
            EventQueryResult {
                timestamp: 1745249390,
                event: EmptyEventEventQueryResult {},
            },
            EventQueryResult {
                timestamp: 1745250830,
                event: EmptyEventEventQueryResult {},
            }
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_query_events_with_only_data_no_filter_no_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .event_with_only_data(
            None,
            None
        )
        .await?;

    let expected_len = 2;
    assert_eq!(result.len(), expected_len);

    assert_eq!(
        result,
        vec![
            EventQueryResult {
                timestamp: 1745249372,
                event: EventWithOnlyDataEventQueryResult {
                    data: EventWithOnlyData {
                        address: Address::from_bech32_string("erd1hdh6zy78ftpsaem8465d973flkawyn5wwv0nu2fsayd6nya9g3ysg9k779")?,
                        amount: BigUint::from_str("560").unwrap(),
                    }
                },
            },
            EventQueryResult {
                timestamp: 1745250848,
                event: EventWithOnlyDataEventQueryResult {
                    data: EventWithOnlyData {
                        address: Address::from_bech32_string("erd1hdh6zy78ftpsaem8465d973flkawyn5wwv0nu2fsayd6nya9g3ysg9k779")?,
                        amount: BigUint::from_str("565").unwrap(),
                    }
                },
            }
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_query_events_with_multi_value_encoded_and_data_no_filter_no_options_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query_events(executor)
        .event_with_multi_value_encoded(
            None,
            None
        )
        .await?;

    let expected_len = 2;
    assert_eq!(result.len(), expected_len);

    assert_eq!(
        result,
        vec![
            EventQueryResult {
                timestamp: 1745249372,
                event: EventWithMultiValueEncodedEventQueryResult {
                    values: vec![],
                    data: BigUint::from(0u8)
                },
            },
            EventQueryResult {
                timestamp: 1745250848,
                event: EventWithMultiValueEncodedEventQueryResult {
                    values: vec![],
                    data: BigUint::from(0u8)
                },
            }
        ]
    );

    Ok(())
}
