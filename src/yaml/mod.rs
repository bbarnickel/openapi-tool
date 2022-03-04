use yaml_rust::parser::Parser;

use self::{model::Node, loader::{Loader, LoaderError}};

mod model;
mod loader;

pub(crate) fn parse_yaml<S: AsRef<str>>(yaml: S) -> Result<Node, LoaderError> {
    let mut parser = Parser::new(yaml.as_ref().chars());
    let mut loader = Loader::new();

    parser.load(&mut loader, false).map_err(|e| {
        let marker = e.marker().clone();
        LoaderError::ScanError(e, marker.into())
    })?;
    loader.get_result()
}

#[cfg(test)]
mod test_parse_yaml {
    use crate::yaml::model::{ScalarNode, ScalarStyle};

    use super::*;

    fn assert_scalar(node: Node, expected_value: &str) {
        assert!(
            matches!(node, Node::Scalar( ScalarNode { ref value, scalar_style: ScalarStyle::Plain,  .. } ) if value == expected_value)
        );
    }

    #[test]
    fn test_empty() {
        let node = parse_yaml("").unwrap();
        assert_scalar(node, "");
    }

    #[test]
    fn test_only_whitespace() {
        let node = parse_yaml("   ").unwrap();
        assert_scalar(node, "");
    }

    #[test]
    fn test_simple_scalar() {
        let node = parse_yaml("Hello World").unwrap();
        assert_scalar(node, "Hello World");
    }

    #[test]
    fn test_simple_seq() {
        let node = parse_yaml(
            r#"- First
- Second
- Third"#,
        )
        .unwrap();
        assert!(matches!(node, Node::Sequence(_)));
        if let Node::Sequence(n) = node {
            assert_eq!(n.value.len(), 3);
            let all_eq = n
                .value
                .iter()
                .map(|n| match n {
                    Node::Scalar(nn) => nn,
                    _ => panic!(),
                })
                .zip(vec!["First", "Second", "Third"])
                .all(|(n, e)| n.value == e);
            assert!(all_eq);
        }
    }
}