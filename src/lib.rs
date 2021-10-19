#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use std::ops::Deref;

/// The `Undo` type wrapping a state that tracks updates and allows undoing or redoing them.
pub struct Undo<TState> {
    /// The initial state used to regenerate the current one.
    initial_state: TState,
    /// The current state to update.
    current_state: TState,
    /// All recorded updates applied to the current state.
    updates: Vec<Box<dyn Fn(&mut TState)>>,
    /// Number of updates applied to the current state. Undoing reduces this number.
    nb_updates: usize,
}

impl<TState: Clone> Undo<TState> {
    /// Wraps the given state in an `Undo`, which will track all updates and allows undoing or redoing them.
    ///
    /// # Example
    /// ```
    /// use simple_undo::Undo;
    ///
    /// let mut wrapper = Undo::new(5);
    /// ```
    pub fn new(state: TState) -> Self {
        Self {
            current_state: state.clone(),
            initial_state: state,
            updates: Vec::new(),
            nb_updates: 0,
        }
    }

    /// Unwraps the inner state to an owned value, disabling the undo/redo feature.
    ///
    /// # Example
    /// ```
    /// # use simple_undo::Undo;
    /// let mut message = Undo::new(String::new());
    /// message.update(|text| text.push_str("Hello "));
    /// message.update(|text| text.push_str("world !"));
    ///
    /// let result: String = message.unwrap();
    /// assert_eq!(result, "Hello world !");
    /// ```
    pub fn unwrap(self) -> TState {
        self.current_state
    }

    /// Updates the current state with the given mutating function.
    ///
    /// Note that future [`Undo::redo`] are reset.
    ///
    /// # Example
    /// ```
    /// # use simple_undo::Undo;
    /// let mut counter = Undo::new(0);
    /// counter.update(|value| *value += 10);
    /// counter.update(|value| *value -= 5);
    /// counter.update(|value| *value += 3);
    /// assert_eq!(*counter, 8);
    /// ```
    pub fn update(&mut self, update_fn: impl Fn(&mut TState) + 'static) {
        if self.nb_updates != self.updates.len() {
            // Discard previous updates when updating after an undo.
            self.updates.truncate(self.nb_updates);
        }
        update_fn(&mut self.current_state);
        self.updates.push(Box::new(update_fn));
        self.nb_updates += 1;
    }

    /// Undo the last update done to the current state.
    ///
    /// # Example
    /// ```
    /// # use simple_undo::Undo;
    /// let mut counter = Undo::new(0);
    /// counter.update(|value| *value += 1);
    /// counter.update(|value| *value += 2);
    /// assert_eq!(*counter, 3);
    ///
    /// counter.undo();
    /// assert_eq!(*counter, 1);
    /// counter.undo();
    /// assert_eq!(*counter, 0);
    /// counter.undo(); // does nothing
    /// assert_eq!(*counter, 0);
    /// ```
    pub fn undo(&mut self) {
        if self.nb_updates == 0 {
            return;
        }
        self.nb_updates -= 1;

        self.current_state = self.initial_state.clone();
        for update_fn in self.updates[..self.nb_updates].iter() {
            update_fn(&mut self.current_state);
        }
    }

    /// Redo the last update that have been undone using [`Undo::undo`].
    ///
    /// # Example
    /// ```
    /// # use simple_undo::Undo;
    /// let mut counter = Undo::new(0);
    /// counter.update(|value| *value += 1); // 1
    /// counter.update(|value| *value += 2); // 3
    /// counter.undo(); // 1
    /// counter.undo(); // 0
    /// assert_eq!(*counter, 0);
    ///
    /// counter.redo();
    /// assert_eq!(*counter, 1);
    /// counter.redo();
    /// assert_eq!(*counter, 3);
    /// counter.redo(); // does nothing
    /// assert_eq!(*counter, 3);
    /// ```
    pub fn redo(&mut self) {
        if self.nb_updates == self.updates.len() {
            return;
        }
        self.updates[self.nb_updates](&mut self.current_state);
        self.nb_updates += 1;
    }
}

impl<TState: Clone> Deref for Undo<TState> {
    type Target = TState;

    fn deref(&self) -> &Self::Target {
        &self.current_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Counter {
        count: u64,
    }

    #[test]
    fn it_can_undo_and_redo_updates() {
        let mut counter = Undo::new(Counter { count: 0 });
        assert_eq!(counter.count, 0);
        counter.update(|c| c.count = 5);
        assert_eq!(counter.count, 5);
        counter.update(|c| c.count += 3);
        assert_eq!(counter.count, 8);

        counter.undo();
        assert_eq!(counter.count, 5);
        counter.undo();
        assert_eq!(counter.count, 0);

        counter.redo();
        assert_eq!(counter.count, 5);
        counter.redo();
        assert_eq!(counter.count, 8);
    }

    #[test]
    fn it_does_nothing_on_too_many_undo_or_redo() {
        let mut counter = Undo::new(Counter { count: 3 });
        counter.undo();
        assert_eq!(counter.count, 3);
        counter.redo();
        assert_eq!(counter.count, 3);
        counter.update(|c| c.count = 8);
        assert_eq!(counter.count, 8);
        counter.undo();
        counter.undo();
        counter.undo();
        assert_eq!(counter.count, 3);
        counter.redo();
        counter.redo();
        counter.redo();
        counter.redo();
        assert_eq!(counter.count, 8);
    }

    #[test]
    fn it_discards_previous_updates_when_updating_after_an_undo() {
        let mut counter = Undo::new(Counter { count: 0 });
        counter.update(|c| c.count += 2);
        counter.update(|c| c.count += 2);
        counter.update(|c| c.count += 2);
        counter.update(|c| c.count += 2);
        counter.update(|c| c.count += 2);
        assert_eq!(counter.count, 10);
        counter.undo(); // 8
        counter.undo(); // 6
        counter.undo(); // 4
        counter.redo(); // 6
        assert_eq!(counter.count, 6);
        counter.update(|c| c.count += 10); // discard previous updates
        assert_eq!(counter.count, 16);
        counter.redo(); // nothing
        counter.redo(); // nothing
        counter.undo(); // 6
        counter.undo(); // 4
        assert_eq!(counter.count, 4);
        counter.redo(); // 6
        counter.redo(); // 16
        counter.redo(); // nothing
        assert_eq!(counter.count, 16);
    }

    #[test]
    fn it_unwraps_the_inner_value() {
        let mut counter = Undo::new(Counter { count: 0 });
        counter.update(|c| c.count = 5);

        let counter: Counter = counter.unwrap();
        assert_eq!(counter.count, 5);
    }

    #[test]
    fn it_works_with_string() {
        let mut input_text = Undo::new(String::new());
        input_text.update(|text| text.push('H')); // H
        input_text.update(|text| text.push('e')); // He
        input_text.update(|text| text.push('l')); // Hel
        input_text.update(|text| text.push('k')); // Helk
        input_text.update(|text| text.push('o')); // Helko
        input_text.undo(); // Helk
        input_text.undo(); // Hel
        input_text.undo(); // He
        input_text.redo(); // Hel
        input_text.update(|text| text.push('l')); // Hell
        input_text.update(|text| text.push('o')); // Hello

        let result: String = input_text.unwrap();
        assert_eq!(result, "Hello");
    }
}
