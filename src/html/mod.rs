mod parser;
mod tag;
mod element;
mod document;

pub use {parser::*, tag::*, element::*, document::*};

#[cfg(test)]
mod test {
    use crate::html::document::Document;
    use crate::html::element::Element;
    use crate::html::tag::Tag;

    #[test]
    fn build_page() {
        let page = Document::new();
        println!("{}", page);
    }

    #[test]
    fn parse_page() {
        let root = Element::Tag(
            Tag::new("html")
                .with_child(
                    Tag::new("head")
                )
                .with_child(
                    Tag::new("body").with_child(
                        Tag::new("h1")
                            .with_id("title")
                            .with_attribute("style", "color: red")
                            .with_text("Hello, this is a test")
                    ))
        );

        let html = "<html><head></head><body><h1 id=\"title\" style=\"color: red\">Hello, this is a test</h1></body></html>".parse::<Element>().unwrap();
        dbg!(&html);
        assert_eq!(root, html);
    }

    #[test]
    fn find_by_id() {
        let html = "<html><head></head><body><h1 id=\"title\" style=\"color: red\">Hello, this is a test</h1></body></html>".parse::<Document>().unwrap();
        let title = html.element_by_id("title").unwrap();
        println!("{:#?}", title.content());
    }
}
