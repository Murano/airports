use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};

#[derive(Clone,Serialize,Deserialize, Debug)]
pub struct Ticket {
    pub id: String,
    pub departure_code:String,
    pub arrival_code:String,
    pub departure_time:i32,
    pub arrival_time:i32,
    pub price:f32
}

impl PartialEq for Ticket {
    fn eq(&self, other: &Ticket) -> bool {
        self.id == other.id
    }
}

impl Eq for Ticket {}

impl Hash for Ticket {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Debug)]
pub struct Airport {
    directions: RefCell<HashSet<Ticket>>
}


impl Airport {
    fn init() -> Airport {
        Airport{
            directions: RefCell::new(HashSet::new())
        }
    }
}

#[derive(Debug)]
pub struct Database {
    pub airports: RefCell<HashMap<String, Airport>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TicketsInsertRequest {
    pub tickets: Vec<Ticket>
}

#[derive(Clone,Serialize,Deserialize)]
pub struct SearchRequest {
    pub departure_code: String,
    pub arrival_code: String,
    pub departure_time_start: i32,
    pub departure_time_end: i32
}

#[derive(Serialize, Debug)]
pub struct Solution {
    ticket_ids: Vec<String>,
    price: f32
}


#[derive(Serialize, Debug)]
pub struct Solutions {
    solutions: Vec<Solution>
}


impl Solutions {
    fn merge(&mut self, mut other: Solutions) {
        self.solutions.append(&mut other.solutions);
    }

    fn is_empty(&self) -> bool {
        self.solutions.is_empty()
    }
}

impl Database {

    pub fn init() -> Database {
        Database{
            airports: RefCell::new(HashMap::new()),
        }
    }

    pub fn insert_tickets(&self, tickets: Vec<Ticket>) -> () {

        for ticket in &tickets {

            match self.airports.borrow_mut().entry(ticket.departure_code.clone()) {
                Entry::Occupied(o) => {
                    o.get().directions.borrow_mut().insert(ticket.to_owned());
                },
                Entry::Vacant(v) => {
                    let airport = Airport::init();
                    airport.directions.borrow_mut().insert(ticket.to_owned());
                    v.insert(airport);
                }
            }

        }
//        println!("Inserted {:?}", self.airports);
    }

    pub fn search_flights<'a>(&self, req: &SearchRequest) -> Result<Solutions, &'a str> {

        match self.airports.borrow().get(&req.departure_code) {
            Some(airport_ref) => {
                let solutions = self.check_tickets(req, airport_ref, vec![]);

                if solutions.is_empty() {
                    Err("Tickets not found")
                } else {
                    Ok(solutions)
                }
            },
            None => Err("airport not found in the list")
        }

    }

    fn check_tickets(&self, req: &SearchRequest, airport: &Airport, mut tickets_chain: Vec<Ticket>) -> Solutions {
        let mut solutions = Solutions { solutions: vec![] };

        //ссылки на тикеты, которые будут учавствовать в поиске в глубину
        for ticket in airport.directions.borrow().iter() {
            if tickets_chain.is_empty() {
                //первый вызов check_tickets

                    if ticket.departure_time < req.departure_time_start || ticket.departure_time > req.departure_time_end {
                        continue; //не входит в промежуток времени
                    }

                    //есть вероятность того что этот билет приведет туда где найдутся варианты для стыковки
                    if ticket.arrival_code != req.arrival_code {
                        if let Some(airport_ref) = self.airports.borrow().get(&ticket.arrival_code) {
                            let solutions_1 = self.check_tickets(req, airport_ref, vec![ticket.clone()]);
                            solutions.merge(solutions_1)
                        }
                    } else {
                        let solution = Solution {
                            ticket_ids: vec![ticket.id.clone()],
                            price: ticket.price
                        };

                        solutions.solutions.push(solution);
                    }
                } else {
                    let chain_length = tickets_chain.len();
                    if chain_length == 5 {
                        break;
                    }


                    let prev_ticket = tickets_chain.last().unwrap().to_owned();

                    //проверяем билет по критерию времени пересадки 3<t<8
                    if prev_ticket.arrival_time + 3 * 60 * 60 > ticket.departure_time  ||
                        ticket.departure_time > prev_ticket.arrival_time + 8 * 60 * 60 {
                        continue; //билет не подходит ни для стыковки ни для дальнейших поисков
                    }

                    //найден билет,который везет в конечную точку
                    if ticket.arrival_code == req.arrival_code {

                        let mut ticket_ids = vec![];
                        let mut price: f32 = 0.0;
                        for ticket_from_chain in tickets_chain.iter() {
                            ticket_ids.push(ticket_from_chain.id.clone());
                            price += ticket_from_chain.price
                        }

                        ticket_ids.push(ticket.id.clone());
                        price += ticket.price;

                        let solution = Solution {
                            ticket_ids,
                            price
                        };

                        solutions.solutions.push(solution);
                        continue;
                    }

                    if let Some(airport_ref) = self.airports.borrow().get(&ticket.arrival_code) {
                        tickets_chain.push(ticket.to_owned());
                        let solutions_1 = self.check_tickets(
                            req, airport_ref, tickets_chain.to_owned()
                        );
                        solutions.merge(solutions_1);
                        tickets_chain.pop();
                    }

            }

        }

        solutions
    }
}