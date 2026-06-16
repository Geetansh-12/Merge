use proptest::prelude::*;
use marked_rs::parse;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 500,
        .. ProptestConfig::default()
    })]

    #[test]
    fn does_not_panic(s in "\\PC*") {
        let _html = parse(&s);
    }
}
