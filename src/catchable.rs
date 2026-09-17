mod generated;

/// See [`bast3st-py.catchable.err`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err)
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct cerr(u32);

impl std::fmt::Debug for cerr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

/*
#[serde(serialize_with = "path")]

Serialize this field using a function that is different from its implementation of Serialize. The given function must be callable as fn<S>(&T, S) -> Result<S::Ok, S::Error> where S: Serializer, although it may also be generic over T. Fields used with serialize_with are not required to implement Serialize.
#[serde(deserialize_with = "path")]

Deserialize this field using a function that is different from its implementation of Deserialize. The given function must be callable as fn<'de, D>(D) -> Result<T, D::Error> where D: Deserializer<'de>, although it may also be generic over T. Fields used with deserialize_with are not required to implement Deserialize.

 * */
