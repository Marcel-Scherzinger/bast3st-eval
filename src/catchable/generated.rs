// GENERATED
use super::cerr;
bitflags::bitflags! {
    impl cerr : u32 {
        /// See [`bast3st-py.catchable.err.network_statusDisallowed`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_statusDisallowed)
        const network_statusDisallowed = 0b1;
        /// See [`bast3st-py.catchable.err.network_respInvalid_notJson`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network_respInvalid_notJson)
        const network_respInvalid_notJson = 0b10;
        /// See [`bast3st-py.catchable.err.network`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.network)
        const network = 0b11;
        /// See [`bast3st-py.catchable.err.missingSelection_input`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection_input)
        const missingSelection_input = 0b100;
        /// See [`bast3st-py.catchable.err.missingSelection_output`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection_output)
        const missingSelection_output = 0b1000;
        /// See [`bast3st-py.catchable.err.missingSelection_variable`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection_variable)
        const missingSelection_variable = 0b10000;
        /// See [`bast3st-py.catchable.err.missingSelection_list_whole`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection_list_whole)
        const missingSelection_list_whole = 0b100000;
        /// See [`bast3st-py.catchable.err.missingSelection_list_item`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection_list_item)
        const missingSelection_list_item = 0b1000000;
        /// See [`bast3st-py.catchable.err.missingSelection_list`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection_list)
        const missingSelection_list = 0b1100000;
        /// See [`bast3st-py.catchable.err.missingSelection`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.missingSelection)
        const missingSelection = 0b1111100;
        /// See [`bast3st-py.catchable.err.regex_syntax`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex_syntax)
        const regex_syntax = 0b10000000;
        /// See [`bast3st-py.catchable.err.regex_noMatch`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex_noMatch)
        const regex_noMatch = 0b100000000;
        /// See [`bast3st-py.catchable.err.regex`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.regex)
        const regex = 0b110000000;
        /// See [`bast3st-py.catchable.err.any`](https://marcel-scherzinger.github.io/bast3st-py/ref_caterr.html#bast3st.catchable.err.any)
        const any = 0b111111111;
    }
}

