// GENERATED
use super::cerr;
bitflags::bitflags! {
    impl cerr : u32 {
        /// See [`bast3st-py.catchable.err.network_statusDisallowed`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_statusDisallowed)
        const network_statusDisallowed = 0b1;
        /// See [`bast3st-py.catchable.err.network_respInvalid_notJson`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_respInvalid_notJson)
        const network_respInvalid_notJson = 0b10;
        /// See [`bast3st-py.catchable.err.network_policy_serverNotAllowed`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_policy_serverNotAllowed)
        const network_policy_serverNotAllowed = 0b100;
        /// See [`bast3st-py.catchable.err.network_external`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_external)
        const network_external = 0b1000;
        /// See [`bast3st-py.catchable.err.network_url_syntax`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_url_syntax)
        const network_url_syntax = 0b10000;
        /// See [`bast3st-py.catchable.err.network`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network)
        const network = 0b11111;
        /// See [`bast3st-py.catchable.err.missing_variable`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missing_variable)
        const missing_variable = 0b100000;
        /// See [`bast3st-py.catchable.err.missing_list`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missing_list)
        const missing_list = 0b1000000;
        /// See [`bast3st-py.catchable.err.missing`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missing)
        const missing = 0b1100000;
        /// See [`bast3st-py.catchable.err.regex_syntax`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex_syntax)
        const regex_syntax = 0b10000000;
        /// See [`bast3st-py.catchable.err.regex_noMatch`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex_noMatch)
        const regex_noMatch = 0b100000000;
        /// See [`bast3st-py.catchable.err.regex_invalidHaystack`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex_invalidHaystack)
        const regex_invalidHaystack = 0b1000000000;
        /// See [`bast3st-py.catchable.err.regex_noGroup`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex_noGroup)
        const regex_noGroup = 0b10000000000;
        /// See [`bast3st-py.catchable.err.regex`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex)
        const regex = 0b11110000000;
        /// See [`bast3st-py.catchable.err.collection_keyInvalidForArray`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.collection_keyInvalidForArray)
        const collection_keyInvalidForArray = 0b100000000000;
        /// See [`bast3st-py.catchable.err.collection_valueNotFound`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.collection_valueNotFound)
        const collection_valueNotFound = 0b1000000000000;
        /// See [`bast3st-py.catchable.err.collection`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.collection)
        const collection = 0b1100000000000;
        /// See [`bast3st-py.catchable.err.typing_notCollection_notArray`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notCollection_notArray)
        const typing_notCollection_notArray = 0b10000000000000;
        /// See [`bast3st-py.catchable.err.typing_notCollection_notMapping`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notCollection_notMapping)
        const typing_notCollection_notMapping = 0b100000000000000;
        /// See [`bast3st-py.catchable.err.typing_notCollection`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notCollection)
        const typing_notCollection = 0b110000000000000;
        /// See [`bast3st-py.catchable.err.typing_notAction`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notAction)
        const typing_notAction = 0b1000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notCriterion`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notCriterion)
        const typing_notCriterion = 0b10000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notValue`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notValue)
        const typing_notValue = 0b100000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notPrimitive`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notPrimitive)
        const typing_notPrimitive = 0b1000000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notText`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notText)
        const typing_notText = 0b10000000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notNumeric`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notNumeric)
        const typing_notNumeric = 0b100000000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notMapkey`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notMapkey)
        const typing_notMapkey = 0b1000000000000000000000;
        /// See [`bast3st-py.catchable.err.typing_notIterable`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing_notIterable)
        const typing_notIterable = 0b10000000000000000000000;
        /// See [`bast3st-py.catchable.err.typing`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.typing)
        const typing = 0b11111111110000000000000;
        /// See [`bast3st-py.catchable.err.any`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.any)
        const any = 0b11111111111111111111111;
    }
}