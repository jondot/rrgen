use crate::{insert_content_at_positions, InsertionPoint, MatchPositions};

#[cfg(test)]
mod test_insert_content_at_positions_tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_insert_content_at_positions_all_after() {
        let input = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        println!("input:{}", input);

        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            input,
            content,
            false,
            &regex,
            MatchPositions::All,
            InsertionPoint::After,
        );

        let expected = r#"
pub struct Hello1 {}
// New content
pub struct Hello2 {}
// New content
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_all_before() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            false,
            &regex,
            MatchPositions::All,
            InsertionPoint::Before,
        );

        let expected = r#"
// New content
pub struct Hello1 {}
// New content
pub struct Hello2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_first_after() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            false,
            &regex,
            MatchPositions::First,
            InsertionPoint::After,
        );

        let expected = r#"
pub struct Hello1 {}
// New content
pub struct Hello2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_first_before() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            false,
            &regex,
            MatchPositions::First,
            InsertionPoint::Before,
        );

        let expected = r#"
// New content
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_last_before() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            false,
            &regex,
            MatchPositions::Last,
            InsertionPoint::Before,
        );

        let expected = r#"
pub struct Hello1 {}
// New content
pub struct Hello2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_last_after() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            false,
            &regex,
            MatchPositions::Last,
            InsertionPoint::After,
        );

        let expected = r#"
pub struct Hello1 {}
pub struct Hello2 {}
// New content
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_inline_first_before() {
        let file_content = r#"
pub struct World1 {}
pub struct World2 {}
"#;
        let content = "Hello";
        let regex = Regex::new(r"World").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            true,
            &regex,
            MatchPositions::First,
            InsertionPoint::Before,
        );

        let expected = r#"
pub struct HelloWorld1 {}
pub struct World2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_inline_first_after() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "World";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            true,
            &regex,
            MatchPositions::First,
            InsertionPoint::After,
        );

        let expected = r#"
pub struct HelloWorld1 {}
pub struct Hello2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_inline_last_before() {
        let file_content = r#"
pub struct World1 {}
pub struct World2 {}
"#;
        let content = "Hello";
        let regex = Regex::new(r"World").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            true,
            &regex,
            MatchPositions::Last,
            InsertionPoint::Before,
        );

        let expected = r#"
pub struct World1 {}
pub struct HelloWorld2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_inline_last_after() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "World";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            true,
            &regex,
            MatchPositions::Last,
            InsertionPoint::After,
        );

        let expected = r#"
pub struct Hello1 {}
pub struct HelloWorld2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_inline_all_after() {
        let file_content = r#"
pub struct Hello1 {}
pub struct Hello2 {}
"#;
        let content = "World";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            true,
            &regex,
            MatchPositions::All,
            InsertionPoint::After,
        );

        let expected = r#"
pub struct HelloWorld1 {}
pub struct HelloWorld2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_inline_all_before() {
        let file_content = r#"
pub struct World1 {}
pub struct World2 {}
"#;
        let content = "Hello";
        let regex = Regex::new(r"World").unwrap();
        let result = insert_content_at_positions(
            file_content,
            content,
            true,
            &regex,
            MatchPositions::All,
            InsertionPoint::Before,
        );

        let expected = r#"
pub struct HelloWorld1 {}
pub struct HelloWorld2 {}
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_content_at_positions_before_last_with_ending_bracket() {
        let input = r#"
pub struct Hello2 {
}"#;
        let content = "// New content";
        let regex = Regex::new(r"Hello").unwrap();
        let result = insert_content_at_positions(
            input,
            content,
            false,
            &regex,
            MatchPositions::First,
            InsertionPoint::Before,
        );

        let expected = r#"
// New content
pub struct Hello2 {
}"#;
        assert_eq!(result, expected);
    }

}
