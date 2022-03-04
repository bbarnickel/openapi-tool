use linked_hash_map_rs::LinkedHashMap;
use yaml_rust::scanner::{Marker, TScalarStyle};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum ScalarStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

impl From<TScalarStyle> for ScalarStyle {
    fn from(style: TScalarStyle) -> Self {
        match style {
            TScalarStyle::SingleQuoted => ScalarStyle::SingleQuoted,
            TScalarStyle::DoubleQuoted => ScalarStyle::DoubleQuoted,
            TScalarStyle::Literal => ScalarStyle::Literal,
            TScalarStyle::Foled => ScalarStyle::Folded,
            _ => ScalarStyle::Plain,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Pos {
    pub(crate) index: usize,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl From<Marker> for Pos {
    fn from(m: Marker) -> Self {
        Self {
            index: m.index(),
            line: m.line(),
            col: m.col(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Node {
    Scalar(ScalarNode),
    Map(MapNode),
    Sequence(SequenceNode),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct ScalarNode {
    pub(crate) value: String,
    pub(crate) pos: Pos,
    pub(crate) scalar_style: ScalarStyle,
}

pub(crate) type Map = LinkedHashMap<ScalarNode, Node>;

#[derive(Debug)]
pub(crate) struct MapNode {
    pub(crate) value: LinkedHashMap<ScalarNode, Node>,
    pub(crate) pos: Pos,
}

impl MapNode {
    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.value.iter().any(|(k, _)| k.value == key)
    }
}

#[derive(Debug)]
pub(crate) struct SequenceNode {
    pub(crate) value: Vec<Node>,
    pub(crate) pos: Pos,
}