//! Shared key-path builder for the encoder.

use std::cell::OnceCell;
use std::rc::Rc;

/// A persistent raw key path used by the encoder.
///
/// The structure shares parent segments and caches its materialized forms so
/// deep traversals avoid repeatedly rebuilding the same key prefixes.
#[derive(Clone, Debug)]
pub(crate) struct KeyPathNode(Rc<KeyPathInner>);

#[derive(Debug)]
struct KeyPathInner {
    parent: Option<KeyPathNode>,
    segment: String,
    depth: usize,
    length: usize,
    materialized: OnceCell<String>,
    dot_encoded: OnceCell<KeyPathNode>,
}

impl KeyPathNode {
    pub(crate) fn from_raw(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let node = Self(Rc::new(KeyPathInner {
            parent: None,
            segment: raw.clone(),
            depth: 1,
            length: raw.len(),
            materialized: OnceCell::new(),
            dot_encoded: OnceCell::new(),
        }));
        let _ = node.0.materialized.set(raw);
        node
    }

    pub(crate) fn append(&self, segment: &str) -> Self {
        if segment.is_empty() {
            return self.clone();
        }

        self.append_owned(segment.to_owned())
    }

    pub(crate) fn append_dot_component(&self, component: &str) -> Self {
        if component.is_empty() {
            return self.clone();
        }

        let mut segment = String::with_capacity(component.len() + 1);
        segment.push('.');
        segment.push_str(component);
        self.append_owned(segment)
    }

    pub(crate) fn append_bracketed_component(&self, component: &str) -> Self {
        let mut segment = String::with_capacity(component.len() + 2);
        segment.push('[');
        segment.push_str(component);
        segment.push(']');
        self.append_owned(segment)
    }

    pub(crate) fn append_empty_list_suffix(&self) -> Self {
        self.append_owned("[]".to_owned())
    }

    pub(crate) fn as_dot_encoded(&self, replacement: &str) -> Self {
        if let Some(cached) = self.0.dot_encoded.get() {
            return cached.clone();
        }

        let mut unresolved = Vec::new();
        let mut current = Some(self.clone());
        let mut base = None;

        while let Some(node) = current {
            if let Some(cached) = node.0.dot_encoded.get() {
                base = Some(cached.clone());
                break;
            }
            unresolved.push(node.clone());
            current = node.0.parent.clone();
        }

        let mut encoded_parent = base;
        for node in unresolved.into_iter().rev() {
            let encoded_segment = if node.0.segment.contains('.') {
                node.0.segment.replace('.', replacement)
            } else {
                node.0.segment.clone()
            };

            let encoded = match (encoded_parent.clone(), node.0.parent.as_ref()) {
                (None, None) => {
                    if encoded_segment == node.0.segment {
                        node.clone()
                    } else {
                        Self::from_raw(encoded_segment)
                    }
                }
                (Some(parent), Some(original_parent))
                    if Rc::ptr_eq(&parent.0, &original_parent.0)
                        && encoded_segment == node.0.segment =>
                {
                    node.clone()
                }
                (Some(parent), _) => parent.append(&encoded_segment),
                (None, Some(_)) => Self::from_raw(encoded_segment),
            };

            let _ = node.0.dot_encoded.set(encoded.clone());
            encoded_parent = Some(encoded);
        }

        self.0
            .dot_encoded
            .get()
            .cloned()
            .unwrap_or_else(|| self.clone())
    }

    pub(crate) fn materialize(&self) -> &str {
        if let Some(cached) = self.0.materialized.get() {
            return cached;
        }

        let mut suffix = Vec::new();
        let mut current = Some(self.clone());
        let mut base = String::new();

        while let Some(node) = current {
            if let Some(cached) = node.0.materialized.get() {
                base = cached.clone();
                break;
            }
            suffix.push(node.clone());
            current = node.0.parent.clone();
        }

        let extra = suffix
            .iter()
            .map(|node| node.0.segment.len())
            .sum::<usize>();
        let mut materialized = String::with_capacity(base.len() + extra);
        materialized.push_str(&base);
        for node in suffix.into_iter().rev() {
            materialized.push_str(&node.0.segment);
        }

        let _ = self.0.materialized.set(materialized);
        self.0
            .materialized
            .get()
            .expect("materialized key path should be cached")
    }

    fn append_owned(&self, segment: String) -> Self {
        Self(Rc::new(KeyPathInner {
            parent: Some(self.clone()),
            length: self.0.length + segment.len(),
            depth: self.0.depth + 1,
            segment,
            materialized: OnceCell::new(),
            dot_encoded: OnceCell::new(),
        }))
    }
}

#[cfg(test)]
mod tests;
