use kernel::common::take_cell::{MapCell};

enum State<'a> {
    NoBuf,
    Buf1(&'a mut [u8]),
    Buf2(&'a mut [u8]),
}

struct TestStruct<'a> {
    state: MapCell<State<'a>>,
    state2: State<'a>,
}

pub fn run() {

    let mut state = State::NoBuf;
    state = match state {
                State::Buf1(buf) => State::Buf2(buf),
                _ => State::NoBuf,
            };
    match state {
        State::NoBuf => { /* ok */ }
        _ => { panic!("Unexpected"); }
    }

    let mut buf = [0; 5];
    let test = TestStruct::new(&mut buf);

    test.process_buf();
    test.process_buf_safer();

    let mut buf2 = [0; 5];
    let mut test2 = TestStruct::new(&mut buf2);
    test2.mutate_state2();
}

impl<'a> TestStruct<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        TestStruct {
            state: MapCell::new(State::Buf1(buf)),
            state2: State::NoBuf,
        }
    }

    fn process_buf(&'a self) {
        match self.state.take() {
            Some(State::Buf1(buf)) => { self.state.put(State::Buf2(buf)); }
            Some(State::Buf2(_)) => { self.state.put(State::NoBuf); }
            Some(State::NoBuf) => { }
            _ => { }
        }
    }

    fn process_buf_safer(&'a self) {
        self.state.modify_owned(move |state| {
            match state {
                Some(State::Buf1(buf)) => Some(State::Buf2(buf)),
                Some(State::Buf2(_)) => Some(State::NoBuf),
                Some(State::NoBuf) => Some(State::NoBuf),
                _ => {
                    // Leave state cell empty
                    None
                }
            }
        });
    }

    fn mutate_state2(&'a mut self) {
        match self.state2 {
            State::Buf1(buf) => { self.state2 = State::Buf2(buf.clone()) }
            State::Buf2(_) => { self.state2 = State::NoBuf }
            _ => { }
        }
    }
}
