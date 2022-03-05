use linked_hash_map_rs::LinkedHashMap;
use yaml_rust::{parser::MarkedEventReceiver, scanner::Marker, Event, ScanError};

use super::model::*;

enum State {
    Initial,
    StreamStarted,
    DocumentStarted,
    MapWaitForKey(Map),
    MapWaitForValue(Map, ScalarNode),
    SequenceWaitForValue(Vec<Node>),
    EndDocument(Node),
    Error(LoaderError),
}

#[derive(Debug)]
pub(crate) enum LoaderError {
    UnexpectedStreamEnd(Pos),
    AliasNotSupported(Pos),
    AnchorNotSupported(Pos),
    TagsNotSupported(Pos),
    KeyNotScalar(Pos),
    DuplicateKey(Pos),
    ScanError(ScanError, Pos),
}

pub(crate) struct Loader {
    stack: Vec<State>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            stack: vec![State::Initial],
        }
    }
}

impl MarkedEventReceiver for Loader {
    fn on_event(&mut self, ev: yaml_rust::Event, mark: Marker) {
        // println!("{:?}", ev);
        let state = self.stack.pop().expect("Invalid state stack!");
        if let State::Error(_) = state {
            self.stack.push(state);
            return;
        }

        let new_state = match ev {
            Event::Nothing => state,
            Event::StreamStart => match state {
                State::Initial => State::StreamStarted,
                _ => unreachable!(),
            },
            Event::StreamEnd => match state {
                State::StreamStarted => {
                    let scalar_node = ScalarNode {
                        value: "".into(),
                        pos: mark.into(),
                        scalar_style: ScalarStyle::Plain,
                    };
                    State::EndDocument(Node::Scalar(scalar_node))
                }
                _ => State::Error(LoaderError::UnexpectedStreamEnd(mark.into())),
            },
            Event::DocumentStart => match state {
                State::StreamStarted => State::DocumentStarted,
                _ => unreachable!(),
            },
            Event::DocumentEnd => match state {
                State::EndDocument(_) => state,
                _ => unreachable!(),
            },
            Event::Alias(_) => State::Error(LoaderError::AliasNotSupported(mark.into())),
            Event::Scalar(value, style, id, tag) => {
                if id == 0 {
                    // no anchor
                    if tag.is_none() {
                        let scalar_node = ScalarNode {
                            value,
                            pos: mark.into(),
                            scalar_style: style.into(),
                        };

                        match state {
                            State::MapWaitForKey(m) => {
                                if m.iter().any(|(k, _)| k.value == scalar_node.value) {
                                    State::Error(LoaderError::DuplicateKey(mark.into()))
                                } else {
                                    State::MapWaitForValue(m, scalar_node)
                                }
                            }
                            State::MapWaitForValue(mut m, k) => {
                                m.insert(k, Node::Scalar(scalar_node));
                                State::MapWaitForKey(m)
                            }
                            State::SequenceWaitForValue(mut s) => {
                                let node = Node::Scalar(scalar_node);
                                s.push(node);
                                State::SequenceWaitForValue(s)
                            }
                            State::DocumentStarted => State::EndDocument(Node::Scalar(scalar_node)),
                            _ => unreachable!(),
                        }
                    } else {
                        State::Error(LoaderError::TagsNotSupported(mark.into()))
                    }
                } else {
                    State::Error(LoaderError::AnchorNotSupported(mark.into()))
                }
            }
            Event::SequenceStart(id) => self.start_block(state, id, mark, false),
            Event::SequenceEnd => match state {
                State::SequenceWaitForValue(s) => {
                    let seq_node = SequenceNode {
                        value: s,
                        pos: mark.into(),
                    };
                    let seq_node = Node::Sequence(seq_node);
                    self.end_block(seq_node)
                }
                _ => unreachable!(),
            },
            Event::MappingStart(id) => self.start_block(state, id, mark, true),
            Event::MappingEnd => match state {
                State::MapWaitForKey(m) => {
                    let map_node = MapNode {
                        value: m,
                        pos: mark.into(),
                    };
                    self.end_block(Node::Map(map_node))
                }
                _ => unreachable!(),
            },
        };

        self.stack.push(new_state);
    }
}

impl Loader {
    fn start_block(&mut self, state: State, id: usize, mark: Marker, is_map: bool) -> State {
        if id == 0 {
            match state {
                State::DocumentStarted
                | State::SequenceWaitForValue(_)
                | State::MapWaitForValue(_, _) => {
                    self.stack.push(state);
                    if is_map {
                        State::MapWaitForKey(LinkedHashMap::new())
                    } else {
                        State::SequenceWaitForValue(vec![])
                    }
                }
                State::MapWaitForKey(_) => State::Error(LoaderError::KeyNotScalar(mark.into())),
                _ => unreachable!(),
            }
        } else {
            State::Error(LoaderError::AnchorNotSupported(mark.into()))
        }
    }

    fn end_block(&mut self, node: Node) -> State {
        let parent = self.stack.pop().expect("Invalid stack");

        match parent {
            State::DocumentStarted => State::EndDocument(node),
            State::MapWaitForValue(mut m, k) => {
                m.insert(k, node);
                State::MapWaitForKey(m)
            }
            State::SequenceWaitForValue(mut s) => {
                s.push(node);
                State::SequenceWaitForValue(s)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_result(mut self) -> Result<Node, LoaderError> {
        let elem = self.stack.pop().expect("Invalid stack");
        match elem {
            State::EndDocument(n) => Ok(n),
            State::Error(e) => Err(e),
            _ => unreachable!(),
        }
    }
}
