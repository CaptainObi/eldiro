use crate::{
    lexer::{Lexeme, SyntaxKind},
    syntax::EldiroLanguage,
};

use super::event::Event;
use rowan::{GreenNode, GreenNodeBuilder, Language};

pub(super) struct Sink<'l, 'input> {
    builder: GreenNodeBuilder<'static>,
    lexemes: &'l [Lexeme<'input>],
    cursor: usize,
    events: Vec<Event<'l>>,
}

impl<'l, 'input> Sink<'l, 'input> {
    pub(super) fn new(lexemes: &'l [Lexeme<'input>], events: Vec<Event<'l>>) -> Self {
        Self {
            builder: GreenNodeBuilder::new(),
            lexemes,
            cursor: 0,
            events,
        }
    }

    pub(super) fn finish(mut self) -> GreenNode {
        let mut reordered_events = self.events.clone();

        for (idx, event) in self.events.iter().enumerate() {
            if let Event::StartNodeAt { kind, checkpoint } = event {
                reordered_events.remove(idx);
                reordered_events.insert(*checkpoint, Event::StartNode { kind: *kind });
            }
        }

        for event in reordered_events {
            match event {
                Event::StartNode { kind } => {
                    self.builder.start_node(EldiroLanguage::kind_to_raw(kind));
                }
                Event::StartNodeAt { .. } => unreachable!(),
                Event::AddToken { kind, text } => self.token(kind, text),
                Event::FinishNode => self.builder.finish_node(),
            }

            self.eat_trivia();
        }

        self.builder.finish()
    }

    fn eat_trivia(&mut self) {
        while let Some(lexeme) = self.lexemes.get(self.cursor) {
            if !lexeme.kind.is_trivia() {
                break;
            }

            self.token(lexeme.kind, lexeme.text);
        }
    }

    fn token(&mut self, kind: SyntaxKind, text: &'l str) {
        self.builder.token(EldiroLanguage::kind_to_raw(kind), text);
        self.cursor += 1;
    }
}
