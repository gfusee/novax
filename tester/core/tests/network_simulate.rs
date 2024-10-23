use std::str::FromStr;
use novax::autoscaleroutercontract::autoscaleroutercontract::{AddLiquidityOperation, AutoscaleRouterContract, EsdtTokenPayment, PoolType, SwapOperation};
use std::sync::Arc;

use async_trait::async_trait;
use hyper::StatusCode;
use num_bigint::BigUint;
use serde::Serialize;
use tokio::sync::Mutex;

use novax::Address;
use novax::errors::NovaXError;
use novax::executor::{BaseSimulationNetworkExecutor, SimulationNetworkExecutor, TokenTransfer};
use novax::tester::tester::TesterContract;
use novax_request::error::request::RequestError;
use novax_request::gateway::client::GatewayClient;

mod utils;

const CALLER: &str = "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g";
const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0";
const AUTOSCALE_ROUTER_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6";

fn get_caller_infos() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"account":{"address":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","nonce":5,"balance":"49893375980000000000","username":"","code":"","codeHash":null,"rootHash":null,"codeMetadata":null,"developerReward":"0","ownerAddress":""},"blockInfo":{"nonce":1514622,"hash":"119621492bad699ac2a60ad276720d1735c1d0eebfe70a82498d8a613a22063a","rootHash":"6ba976a765877a1d9183ca270fc0897ff6b23f30411125243394ed39b309a0b1"}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

fn get_network_config() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"config":{"erd_adaptivity":"false","erd_chain_id":"D","erd_denomination":18,"erd_extra_gas_limit_guarded_tx":50000,"erd_gas_per_data_byte":1500,"erd_gas_price_modifier":"0.01","erd_hysteresis":"0.200000","erd_latest_tag_software_version":"D1.6.6.1","erd_max_gas_per_transaction":600000000,"erd_meta_consensus_group_size":58,"erd_min_gas_limit":50000,"erd_min_gas_price":1000000000,"erd_min_transaction_version":1,"erd_num_metachain_nodes":58,"erd_num_nodes_in_shard":58,"erd_num_shards_without_meta":3,"erd_rewards_top_up_gradient_point":"2000000000000000000000000","erd_round_duration":6000,"erd_rounds_per_epoch":2400,"erd_shard_consensus_group_size":21,"erd_start_time":1694000000,"erd_top_up_factor":"0.500000"}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

fn get_return_caller_simulation_data() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":2384920,"returnMessage":"","smartContractResults":{"4b34385c5a43aa4e2f8b66f63f0e1786aef3e2acff288bd4c2669e71f9078deb":{"nonce":6,"value":26150800000000,"receiver":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","sender":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","data":"@6f6b@e5f5ec2bf6b925565fd1ed99e958858250ce40fd73b12d5792e68bbda679a73c","prevTxHash":"0e888497084b1d6581698537b0a1f1c6f50841fbb5188b5e9b8ebc338662a49f","originalTxHash":"0e888497084b1d6581698537b0a1f1c6f50841fbb5188b5e9b8ebc338662a49f","gasLimit":0,"gasPrice":1000000000,"callType":0,"operation":"transfer","isRefund":true}}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

fn get_return_biguint_argument_simulation_data() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":2442787,"returnMessage":"","smartContractResults":{}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

// I got an error with this transaction when coding the autoscale's API, so I add a dedicated test.
fn get_autoscale_swap_and_deposit_transaction() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":105322372,"returnMessage":"","smartContractResults":{"07bc2a113aee2d326f76e1d52f8e83f9dfd63951e201967478de38f92753458f":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","sender":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","data":"ESDTTransfer@5745474c442d613238633539@03e0eb888875fcfc@6465706f736974","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["279482148534484220"],"operation":"ESDTTransfer","function":"deposit"},"093ccc336aaa04bf3ec189f05a6a33e442e62e82e11dbfdb6b4de278ea9d8244":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"ESDTTransfer@5745474c442d613238633539@fdb286873bde@737761704e6f466565416e64466f7277617264@4d45582d613635396430@0000000000000000000000000000000000000000000000000000000000000000","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["278943203015646"],"operation":"ESDTTransfer","function":"swapNoFeeAndForward"},"1385007ab82c651ef6e535f24623bdabdc4eca98b14910b639249f0116fdd453":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgq5wmxdypdta9m7rlexay0hy26adp3yn9lv5ys7xpyez","sender":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","data":"ESDTTransfer@4845474c442d303737616465@525a61bb@656e7465724d61726b657473@000000000000000005009e0a74f8aee26c518a99ef80734b45cb9cf1991e0463","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["HEGLD-077ade"],"esdtValues":["1381654971"],"operation":"ESDTTransfer","function":"enterMarkets"},"192a2f90292e856f4dd5a03c81256c2d6040eec86be787d480ddf0e80ec63875":{"nonce":0,"value":0,"receiver":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTNFTTransfer@415648534c2d316136386130@92@058e3187b61be341@9fb2d8a743b018eb0539673ecb93f5bc2c653004fe72da7c09016415620971f3","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["AVHSL-1a68a0-92"],"esdtValues":["400311875828179777"],"receivers":["erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g"],"receiversShardIDs":[1],"operation":"ESDTNFTTransfer"},"462ab23c643ad0120b46d94306798907f6480922601620bf641bd25fd83893d5":{"nonce":1,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","data":"ESDTNFTTransfer@415648534c2d316136386130@92@058e3187b61be341@0000000000000000050096bfb81064101023772d1a56d8c411d6c1ae518b0463","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["AVHSL-1a68a0-92"],"esdtValues":["400311875828179777"],"receivers":["erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6"],"receiversShardIDs":[1],"operation":"ESDTNFTTransfer"},"61185fa4c2a0c559058a2f3268d1518ebbdd9fccfe652ccd29793cd2ba69f197":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@555344432d333530633465@982150@73776170546f6b656e734669786564496e707574@5745474c442d613238633539@01","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["9970000"],"operation":"ESDTTransfer","function":"swapTokensFixedInput"},"64ce0b2fa4c2bbd4641ea613280d648ba09d9fbe1a862d3fdbb92cc86e6d214e":{"nonce":26,"value":4946776280000000,"receiver":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"@6f6b@0000000c415648534c2d316136386130000000000000009200000008058e3187b61be341","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"operation":"transfer","isRefund":true},"70f5917bcd5c25b6332ed9bc84d621342e0d81d8de8131581ecd4927d92cbfa4":{"nonce":0,"value":0,"receiver":"erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@555344432d333530633465@7530","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["30000"],"operation":"ESDTTransfer"},"8788e8f387a0dd526807d8c8831c36a5c1711177ce818dab326a9624fc9b2dab":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","data":"swapAndDepositInVault@00000000000000000500b51f2af75f2e9b644f4baf9a5c42d6e2b2e3a6440463@@0000000c5745474c442d6132386335390000000000000000050058137214b0e14c294860a16c11042aa71abc17207ceb0000001473776170546f6b656e734669786564496e707574000000020000000c5745474c442d6132386335390000000101","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":599231000,"gasPrice":1000000000,"callType":0,"operation":"transfer","function":"swapAndDepositInVault"},"c94f5b400257f878e9abde1e0b1e50caadb565e55cb4501734d1dcce0810dd10":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@5745474c442d613238633539@03e0eb888875fcfc@6465706f736974","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["279482148534484220"],"operation":"ESDTTransfer","function":"deposit"},"d254d26255ea33279af46e8491f86a62d536c82899dc8fe318070d66f467ad54":{"nonce":1,"value":279482148534484220,"receiver":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","sender":"erd1qqqqqqqqqqqqqpgqpv09kfzry5y4sj05udcngesat07umyj70n4sa2c0rp","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"originalSender":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","operation":"transfer"},"db2391764eefdc28c4f7b8915c51c2f3b7c0364f7ee19b28edbaa2ae766f85a7":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"ESDTTransfer@5745474c442d613238633539@03e0eb888875fcfc","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["279482148534484220"],"operation":"ESDTTransfer"},"e78d53b5273275359e8e691f14f7f0860a7c76c85971bfffc33c94c4306c74fb":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqpv09kfzry5y4sj05udcngesat07umyj70n4sa2c0rp","sender":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","data":"ESDTTransfer@5745474c442d613238633539@03e0eb888875fcfc@756e7772617045676c64","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["279482148534484220"],"operation":"ESDTTransfer","function":"unwrapEgld"},"fbd5241e55579918b41b70c93fcf6b8d80e427b50b9cf8c6da34f0374b098f5c":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"ESDTTransfer@555344432d333530633465@31@6465706f7369745377617046656573","prevTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","originalTxHash":"f454d9c02fe7cd89504f7629ccab3417c36ebcc571ce41b7110b8cd3e8d20929","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["49"],"operation":"ESDTTransfer","function":"depositSwapFees"}},"logs":{"address":"","events":[{"address":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","mJaA","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFALUfKvdfLptkT0uvmlxC1uKy46ZEBGM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFALUfKvdfLptkT0uvmlxC1uKy46ZEBGM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","dTA=","lq3O+DrwJ9dElYcbuyqGhxlbHdDQXM6Y6WqYvfMRBGM="],"data":"RGlyZWN0Q2FsbA==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","mCFQ","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","MQ==","AAAAAAAAAAAFAHHPwyv1rniWOErl2N5cD4oOE2KyfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz","identifier":"depositSwapFees","topics":["ZGVwb3NpdF9zd2FwX2ZlZXNfZXZlbnQ=","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=","AUo=","AAAAC1VTREMtMzUwYzRlAAAAAAAAAAAAAAABMQ=="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","/bKGhzve","AAAAAAAAAAAFABOe165KoDeS5ryzMjlKQP50bu+kfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g","identifier":"ESDTLocalBurn","topics":["TUVYLWE2NTlkMA==","","Fm109Uj/ZfyD"],"data":null,"additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g","identifier":"swapNoFeeAndForward","topics":["c3dhcF9ub19mZWVfYW5kX2ZvcndhcmQ=","TUVYLWE2NTlkMA==","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=","CUs="],"data":"AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABv2yhoc73gAAAApNRVgtYTY1OWQwAAAACRZtdPVI/2X8gwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABXB5gAAAAAAAAJSwAAAABnCOjo","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","A+DriIh1/Pw=","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","topics":["c3dhcA==","VVNEQy0zNTBjNGU=","V0VHTEQtYTI4YzU5","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM=","CUs="],"data":"AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGMAAAALVVNEQy0zNTBjNGUAAAADmCFQAAAADFdFR0xELWEyOGM1OQAAAAgD4OuIiHX8/AAAAAIm8gAAAAYBJojqQ1sAAAAKB4giMvHTfP6jmwAAAAAAVweYAAAAAAAACUsAAAAAZwjo6A==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","A+DriIh1/Pw=","AAAAAAAAAAAFALUfKvdfLptkT0uvmlxC1uKy46ZEBGM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFAKO2ZpAtX0u/D/k3SPuRWutDEky/ZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFADLeT0Dxei9BxOQMIfWW42qEaZTjZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","A+DriIh1/Pw=","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","A+DriIh1/Pw=","AAAAAAAAAAAFAAseWyRDJQlYSfTjcTRmHVv9zZJefOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqpv09kfzry5y4sj05udcngesat07umyj70n4sa2c0rp","identifier":"ESDTLocalBurn","topics":["V0VHTEQtYTI4YzU5","","A+DriIh1/Pw="],"data":null,"additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqpv09kfzry5y4sj05udcngesat07umyj70n4sa2c0rp","identifier":"transferValueOnly","topics":["A+DriIh1/Pw=","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","identifier":"transferValueOnly","topics":["A+DriIh1/Pw=","AAAAAAAAAAAFADLeT0Dxei9BxOQMIfWW42qEaZTjZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFANyvjUkwheJMt6xVWnEzreLibIwBZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"mintAndEnterMarket","topics":["YWNjcnVlX2ludGVyZXN0X2V2ZW50","BFRfsi7iX3EfVA==","Ab/HfZmjAMM7","DlodCb31UY8=","BJooiwSHq5QuIQ=="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFAKO2ZpAtX0u/D/k3SPuRWutDEky/ZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"ESDTLocalMint","topics":["SEVHTEQtMDc3YWRl","","Ulphuw=="],"data":null,"additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFANyvjUkwheJMt6xVWnEzreLibIwBZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"mintAndEnterMarket","topics":["dXBkYXRlZF9yYXRlc19ldmVudA==","Q8clKw==","FtXQyA=="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"mintAndEnterMarket","topics":["bWludF9ldmVudA==","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM=","A+DriIh1/Pw=","Ulphuw=="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqxt0y7s830gh5r38ypsslt9hrd2zxn98rv5ys0jd2mg","identifier":"ESDTTransfer","topics":["SEVHTEQtMDc3YWRl","","Ulphuw==","AAAAAAAAAAAFAKO2ZpAtX0u/D/k3SPuRWutDEky/ZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgq5wmxdypdta9m7rlexay0hy26adp3yn9lv5ys7xpyez","identifier":"enterMarkets","topics":["c3VwcGxpZXJfcmV3YXJkc19kaXN0cmlidXRlZF9ldmVudA==","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM=","AAAAAQAAAAAAAAAABQAy3k9A8XovQcTkDCH1luNqhGmU42UJAAAAAAtVU0RDLTM1MGM0ZQAAAAUQyt2kwAAAAAUCLZjj5wAAAAoB9s2NgAEVTvmtAAAAD8DI+xfof7lShH6UbmCeBAAAAABlumxCAAAAAGW6bEI=",""],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgq5wmxdypdta9m7rlexay0hy26adp3yn9lv5ys7xpyez","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFAPiDZob3xsDxPXJ1FRt49Z+PL3KaZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgq5wmxdypdta9m7rlexay0hy26adp3yn9lv5ys7xpyez","identifier":"enterMarkets","topics":["ZW50ZXJfbWFya2V0X2V2ZW50","AAAAAAAAAAAFADLeT0Dxei9BxOQMIfWW42qEaZTjZQk=","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM=","Ulphuw=="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFAJ4KdPiu4mxRipnvgHNLRcuc8ZkeBGM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFAKO2ZpAtX0u/D/k3SPuRWutDEky/ZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqnc98f79wufk9rz5ea7q8xj69eww0rxg7q33sfe8uux","identifier":"transferValueOnly","topics":["","AAAAAAAAAAAFADLeT0Dxei9BxOQMIfWW42qEaZTjZQk="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","identifier":"ESDTNFTCreate","topics":["QVZIU0wtMWE2OGEw","kg==","BY4xh7Yb40E=","CAESCQAFjjGHthvjQSI9CJIBGiAAAAAAAAAAAAUAtR8q918um2RPS6+aXELW4rLjpkQEYzIAOhQAAAAAAAAAAAAAAAgFjjGHthvjQQ=="],"data":null,"additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","identifier":"ESDTNFTTransfer","topics":["QVZIU0wtMWE2OGEw","kg==","BY4xh7Yb40E=","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252","identifier":"deposit","topics":["","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM=","CUs="],"data":"AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGMAAAAAAAAAAAUAngp0+K7ibFGKme+Ac0tFy5zxmR4EYwAAAAkNKrQmRc0hVfgAAAAAAAAACAPg64iIdfz8AAAACAWR30z8FPk5AAAACAWOMYe2G+NB","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTNFTTransfer","topics":["QVZIU0wtMWE2OGEw","kg==","BY4xh7Yb40E=","n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM="],"data":"RGlyZWN0Q2FsbA==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"swapAndDepositInVault","topics":["","n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM=","CUs="],"data":"n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfMAAAAAAAAAAAUAtR8q918um2RPS6+aXELW4rLjpkQEYwAAAAtVU0RDLTM1MGM0ZQAAAAAAAAAAAAAAA5iWgAAAAAAAAAACdTAAAAAMV0VHTEQtYTI4YzU5AAAAAAAAAAAAAAAIA+DriIh1/PwAAAAMQVZIU0wtMWE2OGEwAAAAAAAAAJIAAAAIBY4xh7Yb40E=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"completedTxEvent","topics":["9FTZwC/nzYlQT3YpzKs0F8NuvMVxzkG3EQuM0+jSCSk="],"data":null,"additionalData":null}]}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

// I got an error with this transaction when coding the autoscale's API, so I add a dedicated test.
fn get_autoscale_zap_in_xexchange_two_different_tokens_transaction() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":54107445,"returnMessage":"","smartContractResults":{"060454585a1e10af1e2bf4070ef19a63377bd34a297664b3ea69ca26e9b69231":{"nonce":3,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"MultiESDTNFTTransfer@0000000000000000050096bfb81064101023772d1a56d8c411d6c1ae518b0463@02@45474c44555344432d616331613330@@01104638@5745474c442d613238633539@@185cda4cefd8c314","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["EGLDUSDC-ac1a30","WEGLD-a28c59"],"esdtValues":["17843768","1755517978743980820"],"receivers":["erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6"],"receiversShardIDs":[1,1],"operation":"MultiESDTNFTTransfer"},"15327959e5f809006946b8b263b3067b0b4c956a5ab127ca866d223e541ed0c0":{"nonce":1,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"MultiESDTNFTTransfer@0000000000000000050058137214b0e14c294860a16c11042aa71abc17207ceb@02@5745474c442d613238633539@@20b52aecb0a2a646@555344432d333530633465@@0145d537@6164644c6971756964697479@01@01","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59","USDC-350c4e"],"esdtValues":["2356837176062420550","21353783"],"receivers":["erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6"],"receiversShardIDs":[1,1],"operation":"MultiESDTNFTTransfer","function":"addLiquidity"},"4be64a3d232bbfbc073068784a4fc5496694364b99f7e79c43e094f37dd5dba5":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","data":"zapIn@000000000000000000050058137214b0e14c294860a16c11042aa71abc17207ceb000000020000000c5745474c442d6132386335390000000b555344432d333530633465@@0000000b555344432d33353063346500000000000000000500fd96a1cd287f36b0d14c6c47681f8f6b7a89f91152330000000865786368616e6765000000020000000b555344432d33353063346500000001010000000c5745474c442d6132386335390000000000000000050058137214b0e14c294860a16c11042aa71abc17207ceb0000001473776170546f6b656e734669786564496e707574000000020000000c5745474c442d6132386335390000000101@0000000b555344432d33353063346500000000000000000500fd96a1cd287f36b0d14c6c47681f8f6b7a89f91152330000000865786368616e6765000000020000000b555344432d3335306334650000000101","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":598671500,"gasPrice":1000000000,"callType":0,"operation":"transfer","function":"zapIn"},"52cbd53546f5974c421e82efd5f20585acc10dd4377858f38bb8535c02ffc7d2":{"nonce":1,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@555344542d353864356430@4c10a8@65786368616e6765@555344432d333530633465@01","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDT-58d5d0"],"esdtValues":["4985000"],"operation":"ESDTTransfer","function":"exchange"},"53c6472d96ca78dd470fde085c15c11062f29fa25057c083ab593da9413ee45d":{"nonce":2,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","data":"ESDTTransfer@555344432d333530633465@0145d537","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["21353783"],"operation":"ESDTTransfer"},"720cb66ace0745e2b25d02008d4c87d36efadc14d60c6f76d8f0b96f27b25ae4":{"nonce":1,"value":0,"receiver":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@5745474c442d613238633539@185cda4cefd8c314","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["1755517978743980820"],"operation":"ESDTTransfer"},"82c19651ef7bcedb58d6d98109afc396391c6a4ec11a99a9f8141b033a48fc6e":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","data":"ESDTTransfer@555344432d333530633465@0500d76a","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["83941226"],"operation":"ESDTTransfer"},"852816fee5394ffbf44fff6eb36b414c93b9f4c176fbbb69d66be1f552b8fb98":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"ESDTTransfer@555344432d333530633465@01a3@6465706f7369745377617046656573","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["419"],"operation":"ESDTTransfer","function":"depositSwapFees"},"b071257d57201dbcab55c756ebff9dd8bb2acefcaf5cf897191f184b1a63a955":{"nonce":0,"value":0,"receiver":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@45474c44555344432d616331613330@01104638","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["EGLDUSDC-ac1a30"],"esdtValues":["17843768"],"operation":"ESDTTransfer"},"bed88f4050999d8f3299fc35607524046b602de67889216e6e14b07fa1eb58ad":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@555344432d333530633465@0500d76a@73776170546f6b656e734669786564496e707574@5745474c442d613238633539@01","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDC-350c4e"],"esdtValues":["83941226"],"operation":"ESDTTransfer","function":"swapTokensFixedInput"},"cab3cb44951bd6dc3e663680efa05830ddba46a7ddfcdacfdbbbac905cff8478":{"nonce":0,"value":0,"receiver":"erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@555344542d353864356430@7530","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDT-58d5d0"],"esdtValues":["30000"],"operation":"ESDTTransfer"},"d5b5cd111165dd3d0e31bc6694e39081bfcd365548b741f7b8ce0426204908b5":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"ESDTTransfer@5745474c442d613238633539@085b19e846ed02@737761704e6f466565416e64466f7277617264@4d45582d613635396430@0000000000000000000000000000000000000000000000000000000000000000","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["2351966642957570"],"operation":"ESDTTransfer","function":"swapNoFeeAndForward"},"fb9d5f0458b1fc33dddbda4906f66e9cf67131654fc75795dc3af956f175e67c":{"nonce":1,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","data":"ESDTTransfer@5745474c442d613238633539@20b52aecb0a2a646","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["WEGLD-a28c59"],"esdtValues":["2356837176062420550"],"operation":"ESDTTransfer"},"ffefbd4e46fdb0e63320f6a5d975b35dee7901c6ba4ce5fd627feb8aea486023":{"nonce":0,"value":0,"receiver":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","sender":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","data":"ESDTTransfer@555344542d353864356430@4c10a8@65786368616e6765@555344432d333530633465@01","prevTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","originalTxHash":"943e1a01f3458251f4b32d83a10373717ed3cba5be00ba73446d897ba1af2683","gasLimit":0,"gasPrice":1000000000,"callType":0,"tokens":["USDT-58d5d0"],"esdtValues":["4985000"],"operation":"ESDTTransfer","function":"exchange"}},"logs":{"address":"","events":[{"address":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","identifier":"ESDTTransfer","topics":["VVNEVC01OGQ1ZDA=","","mJaA","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["VVNEVC01OGQ1ZDA=","","dTA=","lq3O+DrwJ9dElYcbuyqGhxlbHdDQXM6Y6WqYvfMRBGM="],"data":"RGlyZWN0Q2FsbA==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["VVNEVC01OGQ1ZDA=","","TBCo","AAAAAAAAAAAFAP2Woc0ofzaw0UxsR2gfj2t6ifkRUjM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","BQDXag==","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","identifier":"exchange","topics":["ZXhjaGFuZ2U=","ZwqGOg==","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"AAAAAqQHAAAAAlIDAAAAC1VTRFQtNThkNWQwAAAAA28KEQAAAA0Mnyyc0EZ07epAAAAAAAAAA0wQqAAAAAtVU0RDLTM1MGM0ZQAAAAQyBLZtAAAADQyfLJzQRnTt6kAAAAAAAAAEBQDXag==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","BQDXag==","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","AaM=","AAAAAAAAAAAFAHHPwyv1rniWOErl2N5cD4oOE2KyfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz","identifier":"depositSwapFees","topics":["ZGVwb3NpdF9zd2FwX2ZlZXNfZXZlbnQ=","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=","AUs=","AAAAC1VTREMtMzUwYzRlAAAAAAAAAAAAAAACAaM="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","CFsZ6EbtAg==","AAAAAAAAAAAFABOe165KoDeS5ryzMjlKQP50bu+kfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g","identifier":"ESDTLocalBurn","topics":["TUVYLWE2NTlkMA==","","vUCxHRZc9J5o"],"data":null,"additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g","identifier":"swapNoFeeAndForward","topics":["c3dhcF9ub19mZWVfYW5kX2ZvcndhcmQ=","TUVYLWE2NTlkMA==","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=","CVM="],"data":"AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABwhbGehG7QIAAAAKTUVYLWE2NTlkMAAAAAm9QLEdFlz0nmgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAV0x3AAAAAAAACVMAAAAAZwqGOg==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","ILUq7LCipkY=","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"swapTokensFixedInput","topics":["c3dhcA==","VVNEQy0zNTBjNGU=","V0VHTEQtYTI4YzU5","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM=","CVM="],"data":"AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGMAAAALVVNEQy0zNTBjNGUAAAAEBQDXagAAAAxXRUdMRC1hMjhjNTkAAAAIILUq7LCipkYAAAADAUflAAAABgEmTtmv9QAAAAoHiZ979Hwi+lmKAAAAAABXTHcAAAAAAAAJUwAAAABnCoY6","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["VVNEVC01OGQ1ZDA=","","TBCo","AAAAAAAAAAAFAP2Woc0ofzaw0UxsR2gfj2t6ifkRUjM="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","identifier":"ESDTTransfer","topics":["VVNEQy0zNTBjNGU=","","AUXVNw==","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd","identifier":"exchange","topics":["ZXhjaGFuZ2U=","ZwqGOg==","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"AAAAAim6AAAAAhTdAAAAC1VTRFQtNThkNWQwAAAAA7sauQAAAA0Mnyyc0EZ07epAAAAAAAAAA0wQqAAAAAtVU0RDLTM1MGM0ZQAAAAQwvsxZAAAADQyfLJzQRnTt6kAAAAAAAAAEAUXVNw==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"MultiESDTNFTTransfer","topics":["V0VHTEQtYTI4YzU5","","ILUq7LCipkY=","VVNEQy0zNTBjNGU=","","AUXVNw==","AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="],"data":"RXhlY3V0ZU9uRGVzdENvbnRleHQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"ESDTLocalMint","topics":["RUdMRFVTREMtYWMxYTMw","","ARBGOA=="],"data":null,"additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"MultiESDTNFTTransfer","topics":["RUdMRFVTREMtYWMxYTMw","","ARBGOA==","V0VHTEQtYTI4YzU5","","GFzaTO/YwxQ=","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"QmFja1RyYW5zZmVy","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6","identifier":"addLiquidity","topics":["YWRkX2xpcXVpZGl0eQ==","V0VHTEQtYTI4YzU5","VVNEQy0zNTBjNGU=","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM=","CVM="],"data":"AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGMAAAAMV0VHTEQtYTI4YzU5AAAACAhYUJ/AyeMyAAAAC1VTREMtMzUwYzRlAAAABAFF1TcAAAAPRUdMRFVTREMtYWMxYTMwAAAABAEQRjgAAAAF9e97G9oAAAAKB4mn1EUb48Q8vAAAAAYBJlAfhSwAAAAAAFdMdwAAAAAAAAlTAAAAAGcKhjo=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["RUdMRFVTREMtYWMxYTMw","","ARBGOA==","n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM="],"data":"RGlyZWN0Q2FsbA==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","GFzaTO/YwxQ=","n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM="],"data":"RGlyZWN0Q2FsbA==","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"zapIn","topics":["","n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM=","CVM="],"data":"n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfMAAAALVVNEVC01OGQ1ZDAAAAAAAAAAAAAAAAOYloAAAAACdTAAAAAPRUdMRFVTREMtYWMxYTMwAAAAAAAAAAAAAAAEARBGOAAAAAEAAAAMV0VHTEQtYTI4YzU5AAAAAAAAAAAAAAAIGFzaTO/YwxQ=","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"writeLog","topics":["n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM=","QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk4NjcxNTAwLCBnYXMgdXNlZCA9IDUyOTc4OTQ1"],"data":"QDZmNmJAMDAwMDAwMGY0NTQ3NGM0NDU1NTM0NDQzMmQ2MTYzMzE2MTMzMzAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDQwMTEwNDYzODAwMDAwMDAxMDAwMDAwMGM1NzQ1NDc0YzQ0MmQ2MTMyMzg2MzM1MzkwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDgxODVjZGE0Y2VmZDhjMzE0","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"completedTxEvent","topics":["lD4aAfNFglH0sy2DoQNzcX7Ty6W+ALpzRG2Je6GvJoM="],"data":null,"additionalData":null}]}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

// I got an error with this transaction when coding the autoscale's API, so I add a dedicated test.
fn get_autoscale_zap_in_error_signaled_in_smart_contract() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":0,"returnMessage":"unknown error, code: 12: error signalled by smartcontract","smartContractResults":{},"logs":{"address":"","events":[{"address":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","identifier":"ESDTTransfer","topics":["V0VHTEQtYTI4YzU5","","JxA=","AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM="],"data":"","additionalData":null},{"address":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","identifier":"signalError","topics":["n7LYp0OwGOsFOWc+y5P1vCxlMAT+ctp8CQFkFWIJcfM=","ZXJyb3Igc2lnbmFsbGVkIGJ5IHNtYXJ0Y29udHJhY3Q="],"data":"QDY1Nzg2NTYzNzU3NDY5NmY2ZTIwNjY2MTY5NmM2NTY0","additionalData":null},{"address":"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g","identifier":"internalVMErrors","topics":["AAAAAAAAAAAFAJa/uBBkEBAjdy0aVtjEEdbBrlGLBGM=","emFwSW4="],"data":"CglydW50aW1lLmdvOjg1NiBbZXhlY3V0aW9uIGZhaWxlZF0gW3phcEluXQoJcnVudGltZS5nbzo4NTYgW2V4ZWN1dGlvbiBmYWlsZWRdIFt6YXBJbl0KCXJ1bnRpbWUuZ286ODU2IFtlcnJvciBzaWduYWxsZWQgYnkgc21hcnRjb250cmFjdF0KCXJ1bnRpbWUuZ286ODU2IFtlcnJvciBzaWduYWxsZWQgYnkgc21hcnRjb250cmFjdF0gW3N3YXBUb2tlbnNGaXhlZElucHV0XQoJcnVudGltZS5nbzo4NTYgW2Vycm9yIHNpZ25hbGxlZCBieSBzbWFydGNvbnRyYWN0XSBbc3dhcFRva2Vuc0ZpeGVkSW5wdXRdCglydW50aW1lLmdvOjg1NiBbZXJyb3Igc2lnbmFsbGVkIGJ5IHNtYXJ0Y29udHJhY3RdIFtzd2FwVG9rZW5zRml4ZWRJbnB1dF0KCXJ1bnRpbWUuZ286ODUzIFtTbGlwcGFnZSBleGNlZWRlZF0=","additionalData":null}]}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

struct MockClient {
    url: String
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            url: "".to_string(),
        }
    }
}

#[async_trait]
impl GatewayClient for MockClient {
    type Owned = Self;

    fn get_gateway_url(&self) -> &str {
        &self.url
    }

    fn with_appended_url(&self, url: &str) -> Self::Owned {
        Self {
            url: format!("{}{}", self.url, url),
        }
    }

    async fn get(&self) -> Result<(StatusCode, Option<String>), RequestError> {
        let url = self.get_gateway_url();

        let result = if url == format!("/address/{CALLER}") {
            get_caller_infos()
        } else if url == "/network/config" {
            get_network_config()
        } else {
            unreachable!()
        };

        Ok((result.0, Some(result.1)))
    }

    async fn post<Body>(&self, body: &Body) -> Result<(StatusCode, Option<String>), RequestError> where Body: Serialize + Send + Sync {
        let data = serde_json::to_string(body).unwrap();

        let result = if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"cmV0dXJuQ2FsbGVy","chainId":"D","version":1}"# {
            get_return_caller_simulation_data()
        } else if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"YWRkQDBh","chainId":"D","version":1}"# {
            get_return_biguint_argument_simulation_data()
        } else if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"c3dhcEFuZERlcG9zaXRJblZhdWx0QDAwMDAwMDAwMDAwMDAwMDAwNTAwYjUxZjJhZjc1ZjJlOWI2NDRmNGJhZjlhNWM0MmQ2ZTJiMmUzYTY0NDA0NjNAQDAwMDAwMDBjNTc0NTQ3NGM0NDJkNjEzMjM4NjMzNTM5MDAwMDAwMDAwMDAwMDAwMDA1MDA1ODEzNzIxNGIwZTE0YzI5NDg2MGExNmMxMTA0MmFhNzFhYmMxNzIwN2NlYjAwMDAwMDE0NzM3NzYxNzA1NDZmNmI2NTZlNzM0NjY5Nzg2NTY0NDk2ZTcwNzU3NDAwMDAwMDAyMDAwMDAwMGM1NzQ1NDc0YzQ0MmQ2MTMyMzg2MzM1MzkwMDAwMDAwMTAx","chainId":"D","version":1}"# {
            get_autoscale_swap_and_deposit_transaction()
        } else if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"emFwSW5AMDAwMDAwMDAwMDAwMDAwMDAwMDUwMDU4MTM3MjE0YjBlMTRjMjk0ODYwYTE2YzExMDQyYWE3MWFiYzE3MjA3Y2ViMDAwMDAwMDIwMDAwMDAwYzU3NDU0NzRjNDQyZDYxMzIzODYzMzUzOTAwMDAwMDBiNTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NUBAMDAwMDAwMGI1NTUzNDQ0MzJkMzMzNTMwNjMzNDY1MDAwMDAwMDAwMDAwMDAwMDA1MDBmZDk2YTFjZDI4N2YzNmIwZDE0YzZjNDc2ODFmOGY2YjdhODlmOTExNTIzMzAwMDAwMDA4NjU3ODYzNjg2MTZlNjc2NTAwMDAwMDAyMDAwMDAwMGI1NTUzNDQ0MzJkMzMzNTMwNjMzNDY1MDAwMDAwMDEwMTAwMDAwMDBjNTc0NTQ3NGM0NDJkNjEzMjM4NjMzNTM5MDAwMDAwMDAwMDAwMDAwMDA1MDA1ODEzNzIxNGIwZTE0YzI5NDg2MGExNmMxMTA0MmFhNzFhYmMxNzIwN2NlYjAwMDAwMDE0NzM3NzYxNzA1NDZmNmI2NTZlNzM0NjY5Nzg2NTY0NDk2ZTcwNzU3NDAwMDAwMDAyMDAwMDAwMGM1NzQ1NDc0YzQ0MmQ2MTMyMzg2MzM1MzkwMDAwMDAwMTAxQDAwMDAwMDBiNTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NTAwMDAwMDAwMDAwMDAwMDAwNTAwZmQ5NmExY2QyODdmMzZiMGQxNGM2YzQ3NjgxZjhmNmI3YTg5ZjkxMTUyMzMwMDAwMDAwODY1Nzg2MzY4NjE2ZTY3NjUwMDAwMDAwMjAwMDAwMDBiNTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NTAwMDAwMDAxMDE=","chainId":"D","version":1}"# {
            get_autoscale_zap_in_xexchange_two_different_tokens_transaction()
        } else if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgqj6lmsyryzqgzxaedrftd33q36mq6u5vtq33sp6p0k6","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"RVNEVFRyYW5zZmVyQDU3NDU0NzRjNDQyZDYxMzIzODYzMzUzOUAyNzEwQDdhNjE3MDQ5NmVAMDAwMDAwMDAwMDAwMDAwMDAwMDUwMDU4MTM3MjE0YjBlMTRjMjk0ODYwYTE2YzExMDQyYWE3MWFiYzE3MjA3Y2ViMDAwMDAwMDIwMDAwMDAwYzU3NDU0NzRjNDQyZDYxMzIzODYzMzUzOTAwMDAwMDBiNTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NUBAQDAwMDAwMDBiNTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NTAwMDAwMDAwMDAwMDAwMDAwNTAwNTgxMzcyMTRiMGUxNGMyOTQ4NjBhMTZjMTEwNDJhYTcxYWJjMTcyMDdjZWIwMDAwMDAxNDczNzc2MTcwNTQ2ZjZiNjU2ZTczNDY2OTc4NjU2NDQ5NmU3MDc1NzQwMDAwMDAwMjAwMDAwMDBiNTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NTAwMDAwMDAxMDE=","chainId":"D","version":1}"# {
            get_autoscale_zap_in_error_signaled_in_smart_contract()
        } else {
            unreachable!()
        };

        Ok((result.0, Some(result.1)))
    }
}

fn get_executor() -> Arc<Mutex<BaseSimulationNetworkExecutor<MockClient>>> {
    let executor = BaseSimulationNetworkExecutor::new(
        MockClient::new(),
        Address::from_bech32_string(CALLER).unwrap()
    );

    Arc::new(Mutex::new(executor))
}

// The below test is a success if it compiles
#[tokio::test]
async fn test_clone_simulation_executor() -> Result<(), NovaXError> {
    let executor = SimulationNetworkExecutor::new("".to_string(), Address::from(CALLER));
    #[allow(clippy::redundant_clone)]
    let _executor2 = executor.clone();

    Ok(())
}

// The below test is a success if it compiles
#[tokio::test]
async fn test_debug_network_executor() -> Result<(), NovaXError> {
    let executor = SimulationNetworkExecutor::new("".to_string(), Address::from(CALLER));

    println!("{executor:?}");

    Ok(())
}

#[tokio::test]
async fn test_call_return_caller() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_caller()
        .await?;

    assert!(result.response.is_success());
    assert_eq!(result.result, Some(Address::from_bech32_string(CALLER).unwrap()));

    Ok(())
}

#[tokio::test]
async fn test_call_with_biguint_argument() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    );

    contract
        .call(executor, 600000000)
        .add(&BigUint::from(10u8))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_autoscale_swap_and_deposit_transaction() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = AutoscaleRouterContract::new(
        AUTOSCALE_ROUTER_ADDRESS
    );

    let result = contract
        .call(executor, 600000000)
        .swap_and_deposit_in_vault(
            &Address::from_bech32_string("erd1qqqqqqqqqqqqqpgqk50j4a6l96dkgn6t47d9cskku2ew8fjyq33sjkx252").unwrap(),
            &None,
            &vec![
                SwapOperation {
                    token_out: "WEGLD-a28c59".to_string(),
                    pool_address: Address::from_bech32_string("erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6").unwrap(),
                    function_name: "swapTokensFixedInput".to_string(),
                    arguments: vec![
                        "WEGLD-a28c59".to_string(),
                        String::from_utf8_lossy(&[1]).to_string()
                    ],
                }
            ]
        )
        .await?
        .result
        .unwrap();

    let expected = novax::autoscaleroutercontract::autoscaleroutercontract::EsdtTokenPayment {
        token_identifier: "AVHSL-1a68a0".to_string(),
        token_nonce: 146,
        amount: BigUint::from_str("400311875828179777").unwrap(),
    };

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_autoscale_zap_in_xexchange_two_different_tokens_transaction() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = AutoscaleRouterContract::new(
        AUTOSCALE_ROUTER_ADDRESS
    );

    let result = contract
        .call(executor, 600000000)
        .zap_in(
            &AddLiquidityOperation {
                pool_type: PoolType::XExchange,
                pool_address: "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6".into(),
                tokens: vec![
                    "WEGLD-a28c59".into(),
                    "USDC-350c4e".into(),
                ],
            },
            &None,
            &vec![
                vec![
                    SwapOperation {
                        token_out: "USDC-350c4e".to_string(),
                        pool_address: "erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd".into(),
                        function_name: "exchange".to_string(),
                        arguments: vec![
                            "USDC-350c4e".into(),
                            String::from_utf8_lossy(&[1]).to_string(),
                        ],
                    },
                    SwapOperation {
                        token_out: "WEGLD-a28c59".to_string(),
                        pool_address: "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6".into(),
                        function_name: "swapTokensFixedInput".to_string(),
                        arguments: vec![
                            "WEGLD-a28c59".to_string(),
                            String::from_utf8_lossy(&[1]).to_string(),
                        ],
                    },
                ],
                vec![
                    SwapOperation {
                        token_out: "USDC-350c4e".to_string(),
                        pool_address: "erd1qqqqqqqqqqqqqpgqlkt2rnfg0umtp52vd3rks8u0ddagn7g32ges5p28nd".into(),
                        function_name: "exchange".to_string(),
                        arguments: vec![
                            "USDC-350c4e".to_string(),
                            String::from_utf8_lossy(&[1]).to_string(),
                        ],
                    },
                ],
            ]
        )
        .await?
        .result
        .unwrap();

    let expected = novax::autoscaleroutercontract::autoscaleroutercontract::ZapInResultInfos {
        lp_payment: EsdtTokenPayment {
            token_identifier: "EGLDUSDC-ac1a30".to_string(),
            token_nonce: 0,
            amount: BigUint::from_str("17843768").unwrap(),
        },
        left_payments: vec![
            EsdtTokenPayment {
                token_identifier: "WEGLD-a28c59".to_string(),
                token_nonce: 0,
                amount: BigUint::from_str("1755517978743980820").unwrap()
            }
        ],
    };

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_autoscale_zap_in_error_signaled_by_smart_contract() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = AutoscaleRouterContract::new(
        AUTOSCALE_ROUTER_ADDRESS
    );

    let result = contract
        .call(executor, 600000000)
        .with_esdt_transfers(
            &vec![
                TokenTransfer {
                    identifier: "WEGLD-a28c59".to_string(),
                    nonce: 0,
                    amount: BigUint::from_str("10000").unwrap(),
                }
            ]
        )
        .zap_in(
            &AddLiquidityOperation {
                pool_type: PoolType::XExchange,
                pool_address: "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6".into(),
                tokens: vec![
                    "WEGLD-a28c59".into(),
                    "USDC-350c4e".into(),
                ],
            },
            &None,
            &vec![
                vec![],
                vec![
                    SwapOperation {
                        token_out: "USDC-350c4e".to_string(),
                        pool_address: "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6".into(),
                        function_name: "swapTokensFixedInput".to_string(),
                        arguments: vec![
                            "USDC-350c4e".to_string(),
                            String::from_utf8_lossy(&[1]).to_string(),
                        ],
                    },
                ],
            ]
        )
        .await
        .err()
        .unwrap();

    todo!();

    Ok(())
}

// We don't need more tests for this executor