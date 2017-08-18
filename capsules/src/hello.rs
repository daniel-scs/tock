use kernel::hil;
use kernel::hil::time::{Alarm};

pub struct Hello<'a, A: 'a> {
    alarm: &'a A,
}

impl<'a, A: Alarm> Hello<'a, A> {
    pub fn new(alarm: &'a A) -> Hello<'a, A> {
        Hello { alarm: alarm }
    }

    pub fn start(&self) {
        let delta = 4000;
        let tics = self.alarm.now().wrapping_add(delta);
        self.alarm.set_alarm(tics);
    }
}

impl<'a, A: Alarm> hil::time::Client for Hello<'a, A> {
    fn fired(&self) {
        debug!("Hello, world!");

        self.start();
    }
}
