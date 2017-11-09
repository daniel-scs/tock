use kernel::common::take_cell::{MapCell};

enum State<'a> {
    NoBuf,
    Buf1(&'a mut [u8]),
    Buf2(&'a mut [u8]),
}

struct TestStruct<'a> {
    state: MapCell<State<'a>>,
    state_owned: OwnedCell<State<'a>>,
}

impl<'a> TestStruct<'a> {
    pub fn new(buf: &'a mut [u8], buf_o: &'a mut [u8]) -> Self {
        TestStruct {
            state: MapCell::new(State::Buf1(buf)),
            state_owned: OwnedCell::new(State::Buf1(buf_o)),
        }
    }

    fn step(&self) {
        match self.state.take() {
            Some(State::Buf1(buf)) => { self.state.put(State::Buf2(buf)); }
            Some(State::Buf2(_)) => { self.state.put(State::NoBuf); }
            Some(State::NoBuf) => { }
            _ => { }
        }
    }

    fn step_safer(&self) {
        let what_happened = 
            modify_owned(&self.state, |state| {
                match state {
                    State::Buf1(buf) => (State::Buf2(buf), "Buffered again!"),
                    State::Buf2(_) => (State::NoBuf, "We're done here!"),
                    State::NoBuf => (State::NoBuf, "Horse already dead!"),
                }
            });

        debug!(what_happened);
    }

    fn step_safest(&self) {
        let what_happened = 
            self.state_owned.modify(|state| {
                match state {
                    State::Buf1(buf) => (State::Buf2(buf), "Buffered again!"),
                    State::Buf2(_) => (State::NoBuf, "We're done here!"),
                    State::NoBuf => (State::NoBuf, "Horse already dead!"),
                }
            });

        debug!(what_happened);
    }
}

// A convenience function for a MapCell that tranforms
// its content (also returning a result for the caller),
// assuming nobody has taken it away
fn modify_owned<T, F, R>(cell: &MapCell<T>, f: F) -> R
    where F: FnOnce(T) -> (T, R)
{
    match cell.take() {
        Some(value) => {
            let (value_next, result) = f(value);
            cell.put(value_next);
            result
        },
        None => { panic!("modify_owned: Where my value at?"); }
    }
}

// A wrapper around MapCell that is always full because it doesn't implement `take`
struct OwnedCell<T> {
    value_cell: MapCell<T>
}

impl<T> OwnedCell<T> {

    pub fn new(v: T) -> Self {
        OwnedCell { value_cell: MapCell::new(v) }
    }

    // The transformation function `f` takes the cell's value
    // and produces a new value to store there, as well as a result
    // for the caller.
    pub fn modify<F,R>(&self, f: F) -> R
        where F: FnOnce(T) -> (T, R)
    {
        match self.value_cell.take() {
            Some(value) => {
                let (value_next, result) = f(value);
                self.value_cell.put(value_next);
                result
            },
            None => { panic!("Not reached"); }
        }
    }
}

pub fn run() {
    let mut buf  = [0; 5];
    let mut buf_o = [0; 5];
    let test = TestStruct::new(&mut buf, &mut buf_o);
    test.step();
    test.step_safer();
    test.step_safest();
}
