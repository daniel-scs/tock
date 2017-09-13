use common::take_cell::{TakeCell};
use common::{List, ListNode, ListLink};

pub struct AsyncTakeCell<'a, T: 'a> {
    cell: TakeCell<'a, T>,
    waiting_clients: List<'a, AsyncTakeCellClient<'a, T> + 'a>,
}

impl<'a, T> AsyncTakeCell<'a, T> {
    pub fn new(value: &'a mut T) -> AsyncTakeCell<'a, T> {
        AsyncTakeCell {
            cell: TakeCell::new(value),
            waiting_clients: List::new(),
        }
    }

    pub fn take(&'a self, client: &'a AsyncTakeCellClient<'a, T>) {
        if let Some(value) = self.cell.take() {
            client.taken(value);
        } else {
            self.waiting_clients.push_head(client);
        }
    }

    pub fn replace(&'a self, value: &'a mut T) -> Option<&'a mut T> {
        self.cell.replace(value)
    }
}

pub trait AsyncTakeCellClient<'a, T: 'a> {
    fn taken(&'a self, value: &'a mut T);
    fn next_client(&'a self) -> &'a ListLink<'a, AsyncTakeCellClient<'a, T>>;
}

impl<'a, T> ListNode<'a, AsyncTakeCellClient<'a, T> + 'a> for AsyncTakeCellClient<'a, T> + 'a {
    fn next(&'a self) -> &'a ListLink<'a, AsyncTakeCellClient<'a, T>> {
        self.next_client()
    }
}
