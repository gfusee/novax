multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct TestTokenProperties<M: ManagedTypeApi> {
    pub buffer: ManagedBuffer<M>,
    pub integer: BigUint<M>
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum TestEnumProperties<M: ManagedTypeApi> {
    First,
    Second(ManagedBuffer<M>, BigUint<M>),
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum TestEnumPropertiesWithFields<M: ManagedTypeApi> {
    First { buffer_value: ManagedBuffer<M>, integer: BigUint<M> },
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct CustomStruct<M: ManagedTypeApi> {
    pub first: ManagedBuffer<M>,
    pub second: u64,
    pub third: BigUint<M>
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct CustomStructWithStructAndVec<M: ManagedTypeApi> {
    pub first: ManagedVec<M, u64>,
    pub second: ManagedVec<M, ManagedBuffer<M>>,
    pub third: CustomStruct<M>
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum CustomEnum {
    First,
    Second,
    Third
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum CustomEnumWithValues<M: ManagedTypeApi> {
    First(ManagedBuffer<M>, u64, BigUint<M>),
    Second(ManagedVec<M, u64>, ManagedVec<M, ManagedBuffer<M>>, CustomStruct<M>)
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum CustomEnumWithFields<M: ManagedTypeApi> {
    First { first_first: ManagedBuffer<M>, first_second: u64, first_third: BigUint<M> },
    Second { second_first: ManagedVec<M, u64>, second_second: ManagedVec<M, ManagedBuffer<M>>, second_third: CustomStruct<M> }
}

#[multiversx_sc::module]
pub trait PrinterModule: ContractBase {
    #[endpoint(returnNftProperties)]
    fn return_nft_properties(&self) -> TestTokenProperties<Self::Api> {
        self.blockchain().get_token_attributes(
            &TokenIdentifier::from("NFT-abcdef"),
            6
        )
    }

    #[endpoint(returnNftEnumProperties)]
    fn return_nft_enum_properties(&self) -> TestEnumProperties<Self::Api> {
        self.blockchain().get_token_attributes(
            &TokenIdentifier::from("NFT-abcdef"),
            6
        )
    }

    #[endpoint(returnNftEnumFieldsProperties)]
    fn return_nft_enum_fields_properties(&self) -> TestEnumPropertiesWithFields<Self::Api> {
        self.blockchain().get_token_attributes(
            &TokenIdentifier::from("NFT-abcdef"),
            6
        )
    }

    #[endpoint(returnFungibleBalance)]
    fn return_fungible_balance(&self) -> BigUint<Self::Api> {
        self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from("TEST-abcdef")),
            0
        )
    }

    #[endpoint(returnNonFungibleBalance)]
    fn return_non_fungible_balance(&self) -> BigUint<Self::Api> {
        self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(TokenIdentifier::from("NFT-abcdef")),
            6
        )
    }

    #[endpoint(noArgNoReturnEndpoint)]
    fn no_arg_no_return_endpoint(&self) {}

    #[endpoint(returnCaller)]
    fn return_caller(&self) -> ManagedAddress<Self::Api> {
        self.blockchain().get_caller()
    }

    #[endpoint(returnManagedBuffer)]
    fn return_managed_buffer(&self) -> ManagedBuffer<Self::Api> {
        ManagedBuffer::from("test")
    }

    // there was a bug when an endpoint's parameter is called "contract_address"
    #[endpoint(returnContractAddress)]
    fn return_contract_address(&self, contract_address: ManagedAddress<Self::Api>) -> ManagedAddress<Self::Api> {
        contract_address
    }

    #[endpoint(returnBiguint)]
    fn return_biguint(&self) -> BigUint<Self::Api> {
        BigUint::from(10u8).pow(18)
    }

    #[endpoint(returnU8)]
    fn return_u8(&self) -> u8 {
        3
    }

    #[endpoint(returnU16)]
    fn return_u16(&self) -> u16 {
        500
    }

    #[endpoint(returnU32)]
    fn return_u32(&self) -> u32 {
        200000
    }

    #[endpoint(returnU64)]
    fn return_u64(&self) -> u64 {
        9000000000
    }

    #[endpoint(returnU32Vec)]
    fn return_u32_vec(&self) -> ManagedVec<Self::Api, u32> {
        let mut result = ManagedVec::new();
        result.push(10);
        result.push(200000);

        result
    }

    #[endpoint(returnU64Vec)]
    fn return_u64_vec(&self) -> ManagedVec<Self::Api, u64> {
        let mut result = ManagedVec::new();
        result.push(10);
        result.push(9000000000);

        result
    }

    #[endpoint(returnBufferVec)]
    fn return_buffer_vec(&self) -> ManagedVec<Self::Api, ManagedBuffer<Self::Api>> {
        let mut result = ManagedVec::new();
        result.push(ManagedBuffer::from("test1"));
        result.push(ManagedBuffer::from("test2"));

        result
    }

    #[endpoint(returnBiguintVec)]
    fn return_biguint_vec(&self) -> ManagedVec<Self::Api, BigUint<Self::Api>> {
        let mut result = ManagedVec::new();
        result.push(BigUint::from(10u8).pow(18));
        result.push(BigUint::from(10u8).pow(18) * BigUint::from(2u8));

        result
    }

    #[endpoint(returnTwoU64)]
    fn return_two_u64(&self) -> MultiValue2<u64, u64> {
        MultiValue2::from(
            (
                10,
                9000000000,
            )
        )
    }

    #[endpoint(returnTwoBuffers)]
    fn return_two_buffers(&self) -> MultiValue2<ManagedBuffer<Self::Api>, ManagedBuffer<Self::Api>> {
        MultiValue2::from(
            (
                    ManagedBuffer::from("test1"),
                    ManagedBuffer::from("test2")
                )
        )
    }

    #[endpoint(returnOneBufferOneU64AndOneBiguint)]
    fn return_one_buffer_one_u64_and_one_buffer(&self) -> MultiValue3<ManagedBuffer<Self::Api>, u64, BigUint<Self::Api>> {
        MultiValue3::from(
            (
                ManagedBuffer::from("test1"),
                9000000000,
                BigUint::from(10u8).pow(18)
            )
        )
    }

    #[endpoint(returnDoubleOfU64Arg)]
    fn return_double_of_u64_arg(&self, arg: u64) -> u64 {
        2 * arg
    }

    #[endpoint(returnDoubleOfBiguintArg)]
    fn return_double_of_biguint_arg(&self, arg: BigUint<Self::Api>) -> BigUint<Self::Api> {
        BigUint::from(2u8) * arg
    }

    #[endpoint(returnAppendedBufferArg)]
    fn return_appended_buffer_arg(&self, arg: ManagedBuffer<Self::Api>) -> ManagedBuffer<Self::Api> {
        let mut arg = arg;
        arg.append(&ManagedBuffer::from("test"));

        arg
    }

    #[endpoint(returnSumTwoBiguintArgs)]
    fn return_sum_two_biguint_args(&self, first_arg: BigUint<Self::Api>, second_arg: BigUint<Self::Api>) -> BigUint<Self::Api> {
        first_arg + second_arg
    }

    #[endpoint(returnConcatMultiBufferArgs)]
    fn return_concat_multi_buffer_args(&self, args: MultiValueEncoded<Self::Api, ManagedBuffer<Self::Api>>) -> ManagedBuffer<Self::Api> {
        let mut result = ManagedBuffer::new();
        for arg in args {
            result.append(&arg);
        }

        result
    }

    #[endpoint(returnSumMultiU64Args)]
    fn return_sum_multi_u64_args(&self, args: MultiValueEncoded<Self::Api, u64>) -> u64 {
        let mut result = 0;
        for arg in args {
            result += arg;
        }

        result
    }

    #[endpoint(returnSumMultiBiguintArgs)]
    fn return_sum_multi_biguint_args(&self, args: MultiValueEncoded<Self::Api, BigUint<Self::Api>>) -> BigUint<Self::Api> {
        let mut result = BigUint::zero();
        for arg in args {
            result += arg;
        }

        result
    }

    #[endpoint(returnCustomStruct)]
    fn return_custom_struct(&self) -> CustomStruct<Self::Api> {
        CustomStruct {
            first: ManagedBuffer::from("test"),
            second: 9000000000,
            third: BigUint::from(10u8).pow(18),
        }
    }

    #[endpoint(returnCustomStructArg)]
    fn return_custom_struct_arg(&self, arg: CustomStruct<Self::Api>) -> CustomStruct<Self::Api> {
        arg
    }

    #[endpoint(returnCustomStructWithStructAndVec)]
    fn return_custom_struct_with_struct_and_vec(&self) -> CustomStructWithStructAndVec<Self::Api> {
        let custom_struct = CustomStruct {
            first: ManagedBuffer::from("test"),
            second: 9000000000,
            third: BigUint::from(10u8).pow(18),
        };

        let mut first_vec = ManagedVec::new();
        first_vec.push(10);
        first_vec.push(9000000000);

        let mut second_vec = ManagedVec::new();
        second_vec.push(ManagedBuffer::from("test1"));
        second_vec.push(ManagedBuffer::from("test2"));

        CustomStructWithStructAndVec {
            first: first_vec,
            second: second_vec,
            third: custom_struct
        }
    }

    #[endpoint(returnCustomStructWithStructAndVecArg)]
    fn return_custom_struct_with_struct_and_vec_arg(&self, arg: CustomStructWithStructAndVec<Self::Api>) -> CustomStructWithStructAndVec<Self::Api> {
        arg
    }

    #[endpoint(returnCustomEnum)]
    fn return_custom_enum(&self) -> CustomEnum {
        CustomEnum::Second
    }

    #[endpoint(returnCustomEnumArg)]
    fn return_custom_enum_arg(&self, arg: CustomEnum) -> CustomEnum {
        arg
    }

    #[endpoint(returnFirstCustomEnumWithValues)]
    fn return_first_custom_enum_with_values(&self) -> CustomEnumWithValues<Self::Api> {
        CustomEnumWithValues::First(
            ManagedBuffer::from("test"),
            9000000000,
            BigUint::from(10u8).pow(18),
        )
    }

    #[endpoint(returnSecondCustomEnumWithValues)]
    fn return_second_custom_enum_with_values(&self) -> CustomEnumWithValues<Self::Api> {
        let custom_struct = CustomStruct {
            first: ManagedBuffer::from("test"),
            second: 9000000000,
            third: BigUint::from(10u8).pow(18),
        };

        let mut first_vec = ManagedVec::new();
        first_vec.push(10);
        first_vec.push(9000000000);

        let mut second_vec = ManagedVec::new();
        second_vec.push(ManagedBuffer::from("test1"));
        second_vec.push(ManagedBuffer::from("test2"));

        CustomEnumWithValues::Second(
            first_vec,
            second_vec,
            custom_struct
        )
    }

    #[endpoint(returnCustomEnumWithValuesArg)]
    fn return_custom_enum_with_values_arg(&self, arg: CustomEnumWithValues<Self::Api>) -> CustomEnumWithValues<Self::Api> {
        arg
    }

    #[endpoint(returnFirstCustomEnumWithFields)]
    fn return_first_custom_enum_with_fields(&self) -> CustomEnumWithFields<Self::Api> {
        CustomEnumWithFields::First {
            first_first: ManagedBuffer::from("test"),
            first_second: 9000000000,
            first_third: BigUint::from(10u8).pow(18),
        }
    }

    #[endpoint(returnSecondCustomEnumWithFields)]
    fn return_second_custom_enum_with_fields(&self) -> CustomEnumWithFields<Self::Api> {
        let custom_struct = CustomStruct {
            first: ManagedBuffer::from("test"),
            second: 9000000000,
            third: BigUint::from(10u8).pow(18),
        };

        let mut first_vec = ManagedVec::new();
        first_vec.push(10);
        first_vec.push(9000000000);

        let mut second_vec = ManagedVec::new();
        second_vec.push(ManagedBuffer::from("test1"));
        second_vec.push(ManagedBuffer::from("test2"));

        CustomEnumWithFields::Second {
            second_first: first_vec,
            second_second: second_vec,
            second_third: custom_struct
        }
    }

    #[endpoint(returnCustomEnumWithFieldsArg)]
    fn return_custom_enum_with_fields_arg(&self, arg: CustomEnumWithFields<Self::Api>) -> CustomEnumWithFields<Self::Api> {
        arg
    }

    #[endpoint(returnOptionalValueBool)]
    fn return_optional_value_bool(&self) -> OptionalValue<bool> {
        OptionalValue::Some(true)
    }

    #[endpoint(returnOptionalValueBoolArg)]
    fn return_optional_value_bool_arg(&self, arg: OptionalValue<bool>) -> OptionalValue<bool> {
        arg
    }

    #[endpoint(returnMultiValueTwo)]
    fn return_optional_multi_value_three_arg(&self, arg: OptionalValue<MultiValue3<u64, u64, u64>>) -> OptionalValue<MultiValue3<u64, u64, u64>> {
        arg
    }

    #[endpoint(returnBigIntArg)]
    fn return_bigint_arg(&self, value: BigInt<Self::Api>) -> BigInt<Self::Api> {
        value
    }
}