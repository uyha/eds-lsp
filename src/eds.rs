use tree_sitter::{Node, Parser, Tree, TreeCursor};
use tree_sitter_eds::language;

#[derive(Debug)]
pub struct EDS {
    content: String,
    tree: Tree,
}

impl EDS {
    pub fn parse(content: &str) -> Option<EDS> {
        let mut parser = Parser::new();
        parser.set_language(language()).ok()?;

        Some(EDS {
            content: content.to_owned(),
            tree: parser.parse(content, None)?,
        })
    }
}

#[derive(Debug)]
pub struct EDSNode<'a> {
    pub content: &'a str,
    pub node: Node<'a>,
}
pub struct EDSIterator<'a> {
    content: &'a str,
    cursor: TreeCursor<'a>,
}

impl<'a> IntoIterator for &'a EDS {
    type Item = EDSNode<'a>;
    type IntoIter = EDSIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            content: &self.content,
            cursor: self.tree.walk(),
        }
    }
}

impl<'a> Iterator for EDSIterator<'a> {
    type Item = EDSNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.cursor.node().kind();
        let cursor = &mut self.cursor;

        if (kind == "source_file"
            && cursor.goto_first_child()
            && cursor.node().kind() == "section"
            && cursor.goto_first_child()
            && cursor.goto_next_sibling()
            && cursor.node().kind() == "section_name")
            || (kind == "section_name"
                && cursor.goto_next_sibling()
                && cursor.goto_next_sibling()
                && cursor.node().kind() == "statement")
        {
            let node = cursor.node();
            let content = cursor.node().utf8_text(self.content.as_bytes()).ok()?;

            return Some(Self::Item { content, node });
        } else if kind == "statement" {
            if cursor.goto_next_sibling() && cursor.node().kind() == "statement" {
                let node = cursor.node();
                let content = cursor.node().utf8_text(self.content.as_bytes()).ok()?;

                return Some(Self::Item { content, node });
            }
            if cursor.goto_parent()
                && cursor.goto_next_sibling()
                && cursor.node().kind() == "section"
                && cursor.goto_first_child()
                && cursor.goto_next_sibling()
                && cursor.node().kind() == "section_name"
            {
                let node = cursor.node();
                let content = cursor.node().utf8_text(self.content.as_bytes()).ok()?;

                return Some(Self::Item { content, node });
            }
        }

        None
    }
}
