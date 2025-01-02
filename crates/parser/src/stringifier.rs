use crate::resource_table::StrId;
use crate::veryla_grammar_trait::{HierarchicalIdentifier, Identifier};
use crate::veryla_token::VerylaToken;
use crate::veryla_walker::VerylaWalker;

#[derive(Default)]
pub struct Stringifier {
    string: String,
    ids: Vec<StrId>,
}

impl Stringifier {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn ids(&self) -> &[StrId] {
        &self.ids
    }

    pub fn hierarchical_identifier_with_prefix_suffix(
        &mut self,
        arg: &HierarchicalIdentifier,
        prefix: &Option<String>,
        suffix: &Option<String>,
    ) {
        let list_len = &arg.hierarchical_identifier_list0.len();

        if *list_len == 0 {
            self.identifier_with_prefix_suffix(&arg.identifier, prefix, suffix);
        } else {
            self.identifier(&arg.identifier);
        }

        for x in &arg.hierarchical_identifier_list {
            self.select(&x.select);
        }

        for (i, x) in arg.hierarchical_identifier_list0.iter().enumerate() {
            self.dot(&x.dot);
            if (i + 1) == *list_len {
                self.identifier_with_prefix_suffix(&x.identifier, prefix, suffix);
            } else {
                self.identifier(&x.identifier);
            }
            for x in &x.hierarchical_identifier_list0_list {
                self.select(&x.select);
            }
        }
    }

    fn identifier_with_prefix_suffix(
        &mut self,
        identifier: &Identifier,
        prefix: &Option<String>,
        suffix: &Option<String>,
    ) {
        if prefix.is_some() || suffix.is_some() {
            let token = identifier.identifier_token.append(prefix, suffix);
            self.veryla_token(&token);
        } else {
            self.veryla_token(&identifier.identifier_token);
        }
    }
}

impl VerylaWalker for Stringifier {
    /// Semantic action for non-terminal 'VerylaToken'
    fn veryla_token(&mut self, arg: &VerylaToken) {
        self.string.push_str(&arg.to_string());
        self.ids.push(arg.token.text);
    }
}
