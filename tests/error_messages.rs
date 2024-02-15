#[cfg(test)]
mod test {
    use core::str::FromStr;

    use edn_rs::Edn;

    fn err_as_string(s: &str) -> String {
        let err = Edn::from_str(s).err().unwrap();
        format!("{err:?}")
    }

    #[test]
    fn invalid_keyword() {
        assert_eq!(
            err_as_string(":"),
            "EdnError { code: InvalidKeyword, line: Some(1), column: Some(1), ptr: Some(0) }"
        );
        assert_eq!(
            err_as_string("  :"),
            "EdnError { code: InvalidKeyword, line: Some(1), column: Some(3), ptr: Some(2) }"
        );
        assert_eq!(
            err_as_string("\n\n   :"),
            "EdnError { code: InvalidKeyword, line: Some(3), column: Some(4), ptr: Some(5) }"
        );
    }

    #[test]
    fn unexpected_eof() {
        assert_eq!(
            err_as_string(r#""hello, world!"#),
            "EdnError { code: UnexpectedEOF, line: Some(1), column: Some(15), ptr: Some(14) }"
        );
        assert_eq!(
            err_as_string(
                r#""hello,
multiple
lines
world!"#
            ),
            "EdnError { code: UnexpectedEOF, line: Some(4), column: Some(7), ptr: Some(29) }"
        );
    }

    #[test]
    fn invalid_num() {
        assert_eq!(
            err_as_string(" ,,,, , , ,,,, ,\n ,,,,       0xfoobarlol"),
            "EdnError { code: InvalidNumber, line: Some(2), column: Some(13), ptr: Some(29) }"
        );
        assert_eq!(
            err_as_string("[ ; comment \n-0xfoobarlol 0xsilycat]"),
            "EdnError { code: InvalidNumber, line: Some(2), column: Some(1), ptr: Some(13) }"
        );
        assert_eq!(
            err_as_string("[ ;;;,,,,\n , , ,,,, ,\n ,,,,   16  -0xfoobarlol 0xsilycat]"),
            "EdnError { code: InvalidNumber, line: Some(3), column: Some(13), ptr: Some(34) }"
        );
    }

    #[test]
    fn utf8() {
        let err = Edn::from_str("(猫 ; cat\nおやつ;treats\n      ")
            .err()
            .unwrap();

        assert_eq!(
            format!("{err:?}"),
            "EdnError { code: UnexpectedEOF, line: Some(3), column: Some(7), ptr: Some(34) }"
        );

        assert_eq!(err.line, Some(3));
        assert_eq!(err.column, Some(7));
        assert_eq!(err.ptr, Some(34));
    }

    #[test]
    #[cfg(not(feature = "sets"))]
    fn disabled_features() {
        // Special case of running into a set without the feature enabled
        assert_eq!(
            err_as_string("#{true, \\c, 3,four, }",),
            "EdnError { code: NoFeatureSets, line: Some(1), column: Some(2), ptr: Some(1) }"
        );

        assert_eq!(
            err_as_string("[1 \n2 ;3 \n4 #{true, \\c, 3,four, }]",),
            "EdnError { code: NoFeatureSets, line: Some(3), column: Some(4), ptr: Some(13) }"
        );
    }

    #[test]
    fn invalid_conversions() {
        let small = edn_rs::from_edn::<u8>(&Edn::UInt(9876123));
        assert_eq!(format!("{small:?}"), "Err(EdnError { code: TryFromInt(TryFromIntError(())), line: None, column: None, ptr: None })");
    }
}
