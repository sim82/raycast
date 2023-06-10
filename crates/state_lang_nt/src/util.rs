use cfgrammar::Span;
use lrlex::{DefaultLexerTypes, LRNonStreamingLexer};
use lrpar::NonStreamingLexer;

pub fn remove_comments(input: &mut String) {
    let mut comments = Vec::new();
    let mut potential_start = None;
    let mut start = None;
    for (i, c) in input.char_indices() {
        if c == '/' {
            if start.is_none() && potential_start.is_some() {
                start = potential_start;
            } else {
                potential_start = Some(i);
            }
        } else if c == '\n' {
            if let Some(start) = &start {
                comments.push((*start, i));
            }
            potential_start = None;
            start = None;
        } else {
            potential_start = None;
        }
    }
    comments.reverse();
    for (start, end) in comments {
        input.replace_range(start..end, "");
    }
}
pub trait SpanResolver {
    fn get_span(&self, span: Span) -> &str;
}
impl SpanResolver for LRNonStreamingLexer<'_, '_, DefaultLexerTypes> {
    fn get_span(&self, span: Span) -> &str {
        self.span_str(span)
    }
}
