use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Serialize, Clone)]
pub struct Ticket {
    pub id: u64,
    pub title: String,
}

#[derive(Clone)]
pub struct TicketController {
    pub tickets: Arc<Mutex<Vec<Option<Ticket>>>>,
}

impl TicketController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets: Arc::default(),
        })
    }
}

impl TicketController {
    pub async fn create(&self, title: String) -> Result<Ticket> {
        let mut tickets = self.tickets.lock().unwrap();

        let ticket = Ticket {
            id: tickets.len() as u64,
            title: title,
        };

        tickets.push(Some(ticket.clone()));

        Ok(ticket)
    }

    pub async fn list(&self) -> Result<Vec<Ticket>> {
        let tickets = self.tickets.lock().unwrap();
        let tickets = tickets.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets)
    }

    pub async fn delete(&self, id: u64) -> Result<Ticket> {
        let mut tickets = self.tickets.lock().unwrap();

        let ticket = tickets.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketNotFound { id })
    }
}
