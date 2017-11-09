use kernel::common::take_cell::{MapCell};

enum State {
    NoBuf,
    Buf1(&'static mut [u8]),
    Buf2(&'static mut [u8]),
}

struct TestStruct {
    state: MapCell<State>,
}

impl TestStruct {
    pub fn new() -> TestStruct {
        TestStruct {
            state: MapCell::new(State::NoBuf),
        }
    }

    fn causes_error(&self, buf: &'static mut [u8]) {
        self.state.map(move |state| {
            match *state {
                State::NoBuf => { *state = State::Buf1(buf); },
                State::Buf1(buf) => { *state = State::Buf2(buf); },
                _ => {},
            };
        });
    }
}
