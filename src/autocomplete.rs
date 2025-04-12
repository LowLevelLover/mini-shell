use std::collections::HashMap;

#[derive(Default)]
pub struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end_of_word: bool,
}

impl TrieNode {
    pub fn insert(&mut self, word: &str) {
        let mut current_node = self;
        for ch in word.chars() {
            current_node = current_node.children.entry(ch).or_default();
        }
        current_node.is_end_of_word = true;
    }

    fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let mut completions = Vec::new();
        let mut current_node = self;

        for ch in prefix.chars() {
            if let Some(next_node) = current_node.children.get(&ch) {
                current_node = next_node;
            } else {
                return completions;
            }
        }

        self.dfs(prefix.to_string(), current_node, &mut completions);
        completions
    }

    fn dfs(&self, prefix: String, node: &TrieNode, completions: &mut Vec<String>) {
        if node.is_end_of_word {
            completions.push(prefix.clone());
            return;
        }

        for (ch, child) in &node.children {
            let mut new_prefix = prefix.clone();
            new_prefix.push(*ch);
            child.dfs(new_prefix, child, completions);
        }
    }

    pub fn get_completed_word(&self, word: &str) -> Option<Vec<String>> {
        let result = self.autocomplete(word);

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}
