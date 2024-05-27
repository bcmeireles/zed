use smallvec::SmallVec;

use std::fmt::Debug;

pub enum AstNodeKind {
    Text,

    Bracket,

    Pair,

    UnexpectedClosingBracket,

    List,
}

pub enum AstNode<'a> {
    Pair(PairAstNode<'a>),

    List(ListAstNode<'a>),

    Bracket(BracketAstNode),

    InvalidBracket(InvalidBracketAstNode),

    Text(TextAstNode),
}

impl<'a> AstNode<'a> {
    pub fn kind(&self) -> AstNodeKind {
        match self {
            AstNode::Pair(_) => AstNodeKind::Pair,

            AstNode::List(_) => AstNodeKind::List,

            AstNode::Bracket(_) => AstNodeKind::Bracket,

            AstNode::InvalidBracket(_) => AstNodeKind::UnexpectedClosingBracket,

            AstNode::Text(_) => AstNodeKind::Text,
        }
    }

    pub fn children_length(&self) -> usize {
        match self {
            AstNode::Pair(node) => node.children_length(),

            AstNode::List(node) => node.children_length(),

            _ => 0,
        }
    }

    pub fn get_child(&self, idx: usize) -> Option<&AstNode> {
        match self {
            AstNode::Pair(node) => node.get_child(idx),

            AstNode::List(node) => node.get_child(idx),

            _ => None,
        }
    }

    pub fn children(&self) -> Vec<&AstNode> {
        match self {
            AstNode::Pair(node) => node.children(),

            AstNode::List(node) => node.children(),

            _ => Vec::new(),
        }
    }

    pub fn missing_opening_bracket_ids(&self) -> SmallVec<[usize; 4]> {
        match self {
            AstNode::Pair(node) => node.missing_opening_bracket_ids.clone(),

            AstNode::List(node) => node.missing_opening_bracket_ids.clone(),

            _ => SmallVec::new(),
        }
    }

    pub fn list_height(&self) -> usize {
        match self {
            AstNode::Pair(node) => node.list_height(),

            AstNode::List(node) => node.list_height(),

            _ => 0,
        }
    }

    pub fn length(&self) -> u64 {
        match self {
            AstNode::Pair(node) => node.length,

            AstNode::List(node) => node.length,

            AstNode::Bracket(node) => node.length,

            AstNode::Text(node) => node.length,

            _ => 0,
        }
    }

    pub fn can_be_reused(&self, open_bracket_ids: SmallVec<[usize; 4]>) -> bool {
        match self {
            AstNode::Pair(node) => node.can_be_reused(open_bracket_ids),

            AstNode::List(node) => node.can_be_reused(open_bracket_ids),

            _ => false,
        }
    }

    pub fn flatten_lists(&self) -> &AstNode {
        match self {
            AstNode::Pair(node) => node.flatten_lists(),

            AstNode::List(node) => node.flatten_lists(),

            _ => self,
        }
    }

    pub fn deep_clone(&self) -> AstNode<'a> {
        match self {
            AstNode::Pair(node) => AstNode::Pair(node.deep_clone()),

            AstNode::List(node) => AstNode::List(node.deep_clone()),

            AstNode::Bracket(node) => AstNode::Bracket(node.clone()),

            AstNode::InvalidBracket(node) => AstNode::InvalidBracket(node.clone()),

            AstNode::Text(node) => AstNode::Text(node.clone()),
        }
    }

    pub fn compute_min_indentation(&self, offset: u64, text_model: &impl TextModel) -> usize {
        match self {
            AstNode::Pair(node) => node.compute_min_indentation(offset, text_model),

            AstNode::List(node) => node.compute_min_indentation(offset, text_model),

            _ => usize::MAX,
        }
    }
}

pub trait TextModel {
    fn get_text_range(&self, range: std::ops::Range<usize>) -> &str;

    fn get_line(&self, line_number: usize) -> &str;
}

#[derive(Debug, Clone)]

pub struct PairAstNode<'a> {
    pub length: u64,

    pub opening_bracket: BracketAstNode,

    pub child: Option<&'a AstNode<'a>>, // TODO: Implement debug

    pub closing_bracket: Option<BracketAstNode>,

    pub missing_opening_bracket_ids: SmallVec<[usize; 4]>,
}

impl<'a> PairAstNode<'a> {
    pub fn children_length(&self) -> usize {
        3
    }

    pub fn get_child(&self, idx: usize) -> Option<&AstNode> {
        match idx {
            0 => Some(&AstNode::Bracket(self.opening_bracket.clone())),

            1 => self.child.cloned(), // FIXME: Possible incompatible types

            2 => self
                .closing_bracket
                .as_ref()
                .map(|b| AstNode::Bracket(b.clone())),

            _ => None,
        }
    }

    pub fn children(&self) -> Vec<&AstNode<'a>> {
        let mut result = Vec::new();

        // Push the opening bracket node

        result.push(Box::leak(Box::new(AstNode::Bracket(
            self.opening_bracket.clone(),
        ))));

        // Push the child node if it exists

        if let Some(child) = &self.child {
            result.push(child); // FIXME
        }

        // Push the closing bracket node if it exists

        if let Some(closing_bracket) = &self.closing_bracket {
            result.push(Box::leak(Box::new(AstNode::Bracket(
                closing_bracket.clone(),
            ))));
        }

        result
    }

    pub fn deep_clone(&self) -> PairAstNode<'a> {
        PairAstNode {
            length: self.length,

            opening_bracket: self.opening_bracket.clone(),

            child: self.child,

            closing_bracket: self.closing_bracket.clone(),

            missing_opening_bracket_ids: self.missing_opening_bracket_ids.clone(),
        }
    }

    pub fn compute_min_indentation(&self, offset: u64, text_model: &impl TextModel) -> usize {
        if let Some(child) = &self.child {
            child.compute_min_indentation(offset + self.opening_bracket.length, text_model)
        } else {
            usize::MAX
        }
    }

    pub fn flatten_lists(&self) -> &AstNode {
        self as &AstNode
    }

    pub fn can_be_reused(&self, open_bracket_ids: SmallVec<[usize; 4]>) -> bool {
        self.missing_opening_bracket_ids
            .iter()
            .all(|id| open_bracket_ids.contains(id))
    }

    pub fn list_height(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone)]

pub struct ListAstNode<'a> {
    pub length: u64,

    pub list_height: usize,

    pub missing_opening_bracket_ids: SmallVec<[usize; 4]>,

    pub children: Vec<Box<AstNode<'a>>>, // TODO: Implement debug
}

impl<'a> ListAstNode<'a> {
    pub fn children_length(&self) -> usize {
        self.children.len()
    }

    pub fn get_child(&self, idx: usize) -> Option<&AstNode> {
        self.children.get(idx).map(|child| &**child)
    }

    pub fn children(&self) -> Vec<&AstNode<'a>> {
        self.children.iter().map(|child| &**child).collect()
    }

    pub fn flatten_lists(&self) -> &AstNode {
        self as &AstNode
    }

    pub fn deep_clone(&self) -> ListAstNode<'a> {
        ListAstNode {
            length: self.length,

            list_height: self.list_height,

            missing_opening_bracket_ids: self.missing_opening_bracket_ids.clone(),

            children: self
                .children
                .iter()
                .map(|child| Box::new(child.deep_clone()))
                .collect(),
        }
    }

    pub fn can_be_reused(&self, open_bracket_ids: SmallVec<[usize; 4]>) -> bool {
        self.missing_opening_bracket_ids
            .iter()
            .all(|id| open_bracket_ids.contains(id))
    }

    pub fn compute_min_indentation(&self, offset: u64, text_model: &impl TextModel) -> usize {
        let text = text_model.get_text_range(offset as usize..(offset + self.length) as usize);

        text.lines()
            .map(|line| {
                line.chars()
                    .position(|c| !c.is_whitespace())
                    .unwrap_or(usize::MAX)
            })
            .min()
            .unwrap_or(usize::MAX)
    }

    pub fn list_height(&self) -> usize {
        self.list_height
    }
}

#[derive(Debug, Clone)]

pub struct BracketAstNode {
    pub length: u64,

    pub bracket_type: char,

    pub position: usize,

    pub metadata: Option<String>,
}

#[derive(Debug, Clone)]

pub struct InvalidBracketAstNode {
    pub length: u64,

    pub bracket_type: char,

    pub position: usize,

    pub expected_bracket_type: Option<char>,

    pub metadata: Option<String>,
}

#[derive(Debug, Clone)]

pub struct TextAstNode {
    pub length: u64,
    pub text: String,
}