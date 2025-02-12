#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct Readme;

#[cfg(test)]
mod type_coverage {
    use std::collections::VecDeque;

    use serde::{Deserialize, Serialize};
    use serde_devo::{Devolve, Evolve};

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    struct StructOne {
        #[devo]
        enum_two_private: EnumTwo,
        #[devo]
        pub enum_two_public: EnumTwo,
        #[devo]
        tuple_struct_private: TupleStructTwo,
        #[devo]
        pub tuple_struct_public: TupleStructTwo,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum EnumOne {
        VariantOne,
        VariantTwo {
            struct_field: StructThree,
            #[devo]
            tuple_struct_field: TupleStructTwo,
        },
        #[devo]
        VariantThree(EnumThree),
        #[devo]
        VariantFour(StructThree),
        #[devo]
        VariantFive(TupleStructThree),
    }

    impl Default for EnumOne {
        fn default() -> Self {
            Self::VariantTwo {
                struct_field: StructThree::default(),
                tuple_struct_field: TupleStructTwo::default(),
            }
        }
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    struct TupleStructTwo(#[devo] pub EnumThree, StructThree);

    #[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    struct StructTwo {
        #[devo]
        enum_two_private: EnumThree,
        #[devo]
        pub enum_two_public: EnumThree,
        tuple_struct_private: TupleStructThree,
        pub tuple_struct_public: TupleStructThree,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum EnumTwo {
        VariantOne,
        VariantTwo {
            struct_field: StructThree,
            tuple_struct_field: TupleStructThree,
        },
        #[devo]
        VariantThree(EnumThree),
        #[devo]
        VariantFour(StructThree),
        #[devo]
        VariantFive(TupleStructThree),

        #[serde(untagged)]
        AlreadyUntagged(String),
    }

    impl Default for EnumTwo {
        fn default() -> Self {
            Self::VariantTwo {
                struct_field: StructThree::default(),
                tuple_struct_field: TupleStructThree::default(),
            }
        }
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TupleStructThree(pub String, i64, pub u32, bool);

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct StructThree {
        a_string: String,
        pub an_int: i64,
        another_int: u32,
        pub a_bool: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum EnumThree {
        VariantOne {
            a: i64,
            b: String,
        },
        #[devo]
        VariantTwo(TupleStructThree),
        VariantThree,
    }

    impl Default for EnumThree {
        fn default() -> Self {
            Self::VariantTwo(TupleStructThree::default())
        }
    }

    #[test]
    fn test_roundtrip_direct() {
        let evolved = StructOne::default();
        let evo_json = serde_json::to_string(&evolved).unwrap();
        let devolved: DevolvedStructOne = serde_json::from_str(&evo_json).unwrap();
        let devo_json = serde_json::to_string(&devolved).unwrap();
        assert_eq!(evo_json, devo_json);

        let evo_mp = rmp_serde::to_vec(&evolved).unwrap();
        let devolved: DevolvedStructOne = rmp_serde::from_slice(&evo_mp).unwrap();
        let devo_mp = rmp_serde::to_vec(&devolved).unwrap();
        assert_eq!(evo_mp, devo_mp);

        let mut evo_cbor = VecDeque::new();
        ciborium::into_writer(&evolved, &mut evo_cbor).unwrap();
        let mut devo_cbor = VecDeque::new();
        let devolved: DevolvedStructOne = ciborium::from_reader(&mut evo_cbor.clone()).unwrap();
        ciborium::into_writer(&devolved, &mut devo_cbor).unwrap();
        assert_eq!(evo_cbor, devo_cbor);
    }

    #[test]
    fn test_roundtrip_through_devo() {
        let initial = StructOne::default();
        let evo_json = serde_json::to_string(&initial.clone().into_devolved()).unwrap();
        let devolved: DevolvedStructOne = serde_json::from_str(&evo_json).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        let devo_json = serde_json::to_string(&reevolved.into_devolved()).unwrap();
        let finished = serde_json::from_str::<DevolvedStructOne>(&devo_json)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);

        let evo_mp = rmp_serde::to_vec(&initial.clone().into_devolved()).unwrap();
        let devolved: DevolvedStructOne = rmp_serde::from_slice(&evo_mp).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        let devo_mp = rmp_serde::to_vec(&reevolved.into_devolved()).unwrap();
        let finished = rmp_serde::from_slice::<DevolvedStructOne>(&devo_mp)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);

        let mut rw = VecDeque::new();
        ciborium::into_writer(&initial.clone().into_devolved(), &mut rw).unwrap();
        let devolved: DevolvedStructOne = ciborium::from_reader(&mut rw).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        ciborium::into_writer(&reevolved.into_devolved(), &mut rw).unwrap();
        let finished = ciborium::from_reader::<DevolvedStructOne, _>(&mut rw)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);
    }
}

#[cfg(test)]
mod generic {
    use std::{collections::VecDeque, fmt::Debug};

    use serde::{Deserialize, Serialize};
    use serde_devo::{Devolve, Evolve};

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum FlexibleEnum<T, U, V, W>
    where
        T: Default + Clone + Debug + PartialEq + Devolve,
        <T as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        U: Default + Clone + Debug + PartialEq + Devolve,
        <U as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        V: Default + Clone + Debug + PartialEq,
        W: Default + Clone + Debug + PartialEq,
    {
        #[default]
        UnitVariant,
        NamedVariant {
            #[devo]
            item_one: T,
            #[devo]
            item_two: U,
            item_three: V,
            item_four: W,
        },
        AnonymousVariant(#[devo] T, #[devo] U, V, W),
    }

    impl<T, U, V, W> FlexibleEnum<T, U, V, W>
    where
        T: Default + Clone + Debug + PartialEq + Devolve,
        <T as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        U: Default + Clone + Debug + PartialEq + Devolve,
        <U as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        V: Default + Clone + Debug + PartialEq,
        W: Default + Clone + Debug + PartialEq,
    {
        fn named() -> Self {
            Self::NamedVariant {
                item_one: Default::default(),
                item_two: Default::default(),
                item_three: Default::default(),
                item_four: Default::default(),
            }
        }

        fn anonymous() -> Self {
            Self::AnonymousVariant(
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            )
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum NewFlexibleEnum<T, U, V, W>
    where
        T: Default + Clone + Debug + PartialEq + Devolve,
        <T as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        U: Default + Clone + Debug + PartialEq + Devolve,
        <U as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        V: Default + Clone + Debug + PartialEq,
        W: Default + Clone + Debug + PartialEq,
    {
        UnitVariant,
        NamedVariant {
            #[devo]
            item_one: T,
            #[devo]
            item_two: U,
            item_three: V,
            item_four: W,
        },
        #[devo]
        AnonymousVariant(#[devo] T, #[devo] U, V, W),
        NewVariant(T, U, V, W),
    }

    impl<T, U, V, W> Default for NewFlexibleEnum<T, U, V, W>
    where
        T: Default + Clone + Debug + PartialEq + Devolve,
        <T as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        U: Default + Clone + Debug + PartialEq + Devolve,
        <U as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        V: Default + Clone + Debug + PartialEq,
        W: Default + Clone + Debug + PartialEq,
    {
        fn default() -> Self {
            Self::NewVariant(
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            )
        }
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    struct FlexibleStruct<T, U, V, W>
    where
        T: Default + Clone + Debug + PartialEq + Devolve,
        <T as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        U: Default + Clone + Debug + PartialEq + Devolve,
        <U as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        V: Default + Clone + Debug + PartialEq,
        W: Default + Clone + Debug + PartialEq,
    {
        #[devo]
        item_one: T,
        #[devo]
        item_two: U,
        item_three: V,
        item_four: W,
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    struct FlexibleTupleStruct<T, U, V, W>(#[devo] T, #[devo] U, V, W)
    where
        T: Default + Clone + Debug + PartialEq + Devolve,
        <T as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        U: Default + Clone + Debug + PartialEq + Devolve,
        <U as Devolve>::Devolved: for<'a> Deserialize<'a> + Serialize,
        V: Default + Clone + Debug + PartialEq,
        W: Default + Clone + Debug + PartialEq;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum ConcreteEnum {
        MyVariant { foo: String, bar: i32 },
    }

    impl Default for ConcreteEnum {
        fn default() -> Self {
            Self::MyVariant {
                foo: "abc".to_string(),
                bar: 123,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Devolve)]
    enum NewEnum {
        MyVariant { foo: String, bar: i32 },
        MyNewVariant { baz: bool, corge: (i32, i32) },
    }

    impl Default for NewEnum {
        fn default() -> Self {
            Self::MyNewVariant {
                baz: false,
                corge: (456, 789),
            }
        }
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ConcreteStruct {
        foo: String,
        bar: i32,
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ConcreteTupleStruct(String, i32);

    type MyVeryComplexType = FlexibleEnum<
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleTupleStruct<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
    >;
    type MyVeryComplexDevolvedType = DevolvedFlexibleEnum<
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleTupleStruct<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
    >;
    type MyVeryComplexTypeWithChanges = FlexibleEnum<
        NewFlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleEnum<NewEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleTupleStruct<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
        FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
    >;

    #[test]
    fn test_roundtrip_direct() {
        let initial: MyVeryComplexType = FlexibleEnum::AnonymousVariant(
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::named(),
            FlexibleTupleStruct::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::anonymous(),
        );
        let evo_json = serde_json::to_string(&initial.clone().into_devolved()).unwrap();
        let devolved: MyVeryComplexDevolvedType = serde_json::from_str(&evo_json).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        let devo_json = serde_json::to_string(&reevolved.into_devolved()).unwrap();
        let finished = serde_json::from_str::<MyVeryComplexDevolvedType>(&devo_json)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);

        let evo_mp = rmp_serde::to_vec(&initial.clone().into_devolved()).unwrap();
        let devolved: MyVeryComplexDevolvedType = rmp_serde::from_slice(&evo_mp).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        let devo_mp = rmp_serde::to_vec(&reevolved.into_devolved()).unwrap();
        let finished = rmp_serde::from_slice::<MyVeryComplexDevolvedType>(&devo_mp)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);

        let mut rw = VecDeque::new();
        ciborium::into_writer(&initial.clone().into_devolved(), &mut rw).unwrap();
        let devolved: MyVeryComplexDevolvedType = ciborium::from_reader(&mut rw).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        ciborium::into_writer(&reevolved.into_devolved(), &mut rw).unwrap();
        let finished = ciborium::from_reader::<MyVeryComplexDevolvedType, _>(&mut rw)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);
    }

    #[test]
    fn test_roundtrip_through_devo() {
        let initial: MyVeryComplexType = FlexibleEnum::AnonymousVariant(
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::named(),
            FlexibleTupleStruct::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::anonymous(),
        );
        let evo_json = serde_json::to_string(&initial.clone().into_devolved()).unwrap();
        let devolved: MyVeryComplexDevolvedType = serde_json::from_str(&evo_json).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        let devo_json = serde_json::to_string(&reevolved.into_devolved()).unwrap();
        let finished = serde_json::from_str::<MyVeryComplexDevolvedType>(&devo_json)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);

        let evo_mp = rmp_serde::to_vec(&initial.clone().into_devolved()).unwrap();
        let devolved: MyVeryComplexDevolvedType = rmp_serde::from_slice(&evo_mp).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        let devo_mp = rmp_serde::to_vec(&reevolved.into_devolved()).unwrap();
        let finished = rmp_serde::from_slice::<MyVeryComplexDevolvedType>(&devo_mp)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);

        let mut rw = VecDeque::new();
        ciborium::into_writer(&initial.clone().into_devolved(), &mut rw).unwrap();
        let devolved: MyVeryComplexDevolvedType = ciborium::from_reader(&mut rw).unwrap();
        let reevolved = devolved.try_into_evolved().unwrap();
        ciborium::into_writer(&reevolved.into_devolved(), &mut rw).unwrap();
        let finished = ciborium::from_reader::<MyVeryComplexDevolvedType, _>(&mut rw)
            .unwrap()
            .try_into_evolved()
            .unwrap();
        assert_eq!(initial, finished);
    }

    #[test]
    fn test_compat() {
        let initial: MyVeryComplexTypeWithChanges = FlexibleEnum::AnonymousVariant(
            NewFlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
            FlexibleEnum::<NewEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::named(),
            FlexibleTupleStruct::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
            FlexibleEnum::<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>::default(),
        );
        let mut evo_cbor = VecDeque::new();
        ciborium::into_writer(&initial.clone().into_devolved(), &mut evo_cbor).unwrap();
        let evo_rmp = rmp_serde::to_vec(&initial).unwrap();
        let evo_json = serde_json::to_string(&initial).unwrap();

        let devo_cbor: MyVeryComplexDevolvedType = ciborium::from_reader(&mut evo_cbor).unwrap();
        let devo_rmp: MyVeryComplexDevolvedType = rmp_serde::from_slice(&evo_rmp).unwrap();
        let devo_json: MyVeryComplexDevolvedType = serde_json::from_str(&evo_json).unwrap();

        for deser in [devo_cbor, devo_rmp, devo_json] {
            assert!(match deser {
                DevolvedFlexibleEnum::<
                    FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
                    FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
                    FlexibleTupleStruct<
                        ConcreteEnum,
                        ConcreteEnum,
                        ConcreteStruct,
                        ConcreteTupleStruct,
                    >,
                    FlexibleEnum<ConcreteEnum, ConcreteEnum, ConcreteStruct, ConcreteTupleStruct>,
                >::AnonymousVariant(
                    DevolvedFlexibleEnum::UnrecognizedVariant(_),
                    DevolvedFlexibleEnum::NamedVariant {
                        item_one: DevolvedConcreteEnum::UnrecognizedVariant(_),
                        ..
                    },
                    _,
                    _,
                ) => true,
                _ => false,
            });
        }
    }
}

#[cfg(test)]
mod fallback {
    use std::collections::VecDeque;

    use serde::{Deserialize, Serialize};
    use serde_devo::Devolve;

    #[derive(Serialize, Deserialize, Devolve)]
    #[devo(fallback = serde_json::Value)]
    enum MyJsonEnum {
        Foo,
        Bar,
        Baz,
    }

    #[derive(Serialize, Deserialize, Devolve)]
    #[devo(fallback = ciborium::Value)]
    enum MyCborEnum {
        Foo,
        Bar,
        Baz,
    }

    #[derive(Serialize, Deserialize, Devolve)]
    #[devo(fallback = (bool, bool, bool))]
    enum MyBrokenEnum {
        Foo,
        Bar,
        Baz,
    }

    #[derive(Serialize, Deserialize, Devolve)]
    pub enum MyNewEnum {
        Foo,
        Bar,
        Baz,
        Qux,
        Corge,
    }

    #[test]
    fn test_fallback() {
        let data = MyNewEnum::Corge;
        let mut cbor = VecDeque::new();
        ciborium::into_writer(&data, &mut cbor).unwrap();
        let json = serde_json::to_string(&data).unwrap();

        let _from_cbor =
            ciborium::from_reader::<DevolvedMyCborEnum, &mut VecDeque<u8>>(&mut cbor).unwrap();
        let _from_json = serde_json::from_str::<DevolvedMyJsonEnum>(&json).unwrap();

        assert!(serde_json::from_str::<DevolvedMyBrokenEnum>(&json).is_err());
    }
}
