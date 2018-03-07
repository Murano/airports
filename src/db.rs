use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cell::RefCell;

#[derive(Clone,Serialize,Deserialize, Debug)]
pub struct Ticket {
    id: String,
    departure_code:String,
    arrival_code:String,
    departure_time:i32,
    arrival_time:i32,
    price:f32
}

#[derive(Debug)]
pub struct Airport {
    directions: RefCell<Vec<Ticket>>
}


impl Airport {
    fn init() -> Airport {
        Airport{
            directions: RefCell::new(vec![])
        }
    }
}

#[derive(Debug)]
pub struct Database {
    pub airports: RefCell<HashMap<String, Airport>>
}

#[derive(Deserialize)]
pub struct TicketsInsertRequest {
    pub tickets: Vec<Ticket>
}

#[derive(Clone,Serialize,Deserialize)]
pub struct SearchRequest {
    departure_code: String,
    arrival_code: String,
    departure_time_start: i32,
    departure_time_end: i32
}

#[derive(Serialize)]
pub struct Solution {
    ticket_ids: RefCell<Vec<String>>,
    price: f32
}


#[derive(Serialize)]
pub struct Solutions {
    solutions: RefCell<Vec<Solution>>
}


impl Solutions {
    fn merge(&self, other: Solutions) {
        self.solutions.borrow_mut().append(&mut other.solutions.borrow_mut());
    }

    fn is_empty(&self) -> bool {
        self.solutions.borrow().is_empty()
    }
}

impl Database {

    pub fn insert_tickets(&self, tickets: Vec<Ticket>) -> () {
        for ticket in &tickets {
            match self.airports.borrow_mut().entry(ticket.arrival_code.clone()) {
                Entry::Occupied(o) => o.get().directions.borrow_mut().push(ticket.to_owned()),
                Entry::Vacant(v) => {
                    let airport = Airport::init();
                    airport.directions.borrow_mut().push(ticket.to_owned());
                    v.insert(airport);
                }
            }

        }
//        println!("{:?}", self.airports);
    }

    pub fn search_flights<'a>(&self, req: &SearchRequest) -> Result<Solutions, &'a str> {
//        println!("{:?}", self.airports);
        match self.airports.borrow().get(&req.departure_code) {
            Some(airport_ref) => {
                let solutions = self.check_tickets(req, airport_ref, None);

                if solutions.is_empty() {
                    Err("Tickets not found")
                } else {
                    Ok(solutions)
                }
            },
            None => Err("airport not found in the list")
        }

    }

    fn check_tickets(&self, req: &SearchRequest, airport: &Airport, opt_rc_tickets_chain: Option<RefCell<Vec<Ticket>>>) -> Solutions {
        let solutions = Solutions { solutions: RefCell::new(vec![]) };
        //ссылки на тикеты, которые будут учавствовасть в поиске в глубину

        for ticket in airport.directions.borrow().iter() {
            match opt_rc_tickets_chain {
                //первый вызов check_tickets
                None => {

                    if ticket.departure_time < req.departure_time_start || ticket.departure_time > req.departure_time_end {
                        continue; //не входит в промежуток времени
                    }

                    //есть вероятность того что этот билет приведет туда где найдутся варианты для стыковки
                    if ticket.arrival_code != req.arrival_code {
                        if let Some(airport_ref) = self.airports.borrow().get(&ticket.departure_code) {
                            let solutions_1 = self.check_tickets(req, airport_ref, Some(RefCell::new(vec![ticket.clone()])));
                            solutions.merge(solutions_1)
                        }
                    } else {
                        let solution = Solution {
                            ticket_ids: RefCell::new(vec![ticket.id.clone()]),
                            price: ticket.price
                        };

                        solutions.solutions.borrow_mut().push(solution);
                    }
                }
                //ради упрощения сделан поиск максимум с пятью стыковками
                Some(ref tickets_chain) => {
                    let chain_length = tickets_chain.borrow().len();
                    if chain_length == 5 {
                        break;
                    }

                    //проверяем билет по критерию времени пересадки 3<t<8

                    let last_ticket = tickets_chain.borrow().last().unwrap().to_owned();

                    if last_ticket.arrival_time + 3 * 60 * 60 > ticket.departure_time  || ticket.departure_time > last_ticket.arrival_time + 3 * 60 * 60 {
                        continue; //билет не подходит ни для стыковки ни для дальнейших поисков
                    }

                    //найден билет,который везет в конечную точку
                    if ticket.arrival_code == req.arrival_code {

                        let mut ticket_ids = vec![];
                        let mut price: f32 = 0.0;
                        for ticket_from_chain in tickets_chain.borrow().iter() {
                            ticket_ids.push(ticket_from_chain.id.clone());
                            price += ticket_from_chain.price
                        }

                        ticket_ids.push(ticket.id.clone());
                        price += ticket.price;

                        let solution = Solution {
                            ticket_ids: RefCell::new(ticket_ids),
                            price
                        };

                        solutions.solutions.borrow_mut().push(solution);
                        continue;
                    }

                    if let Some(airport_ref) = self.airports.borrow().get(&ticket.departure_code) {
                        tickets_chain.borrow_mut().push(ticket.to_owned());
                        let solutions_1 = self.check_tickets(
                            req, airport_ref, Some(RefCell::new(tickets_chain.borrow_mut().to_owned()))
                        );
                        solutions.merge(solutions_1)
                    }
                }
            }

        }

        solutions
    }
}