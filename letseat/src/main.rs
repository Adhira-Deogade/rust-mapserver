use std::thread;
use std::time::Duration;
use std::io::{Write, prelude::*, BufReader};
use std::thread::sleep;
use std::sync::mpsc::{self, Sender, Receiver};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use rand::Rng;
use serde::Serialize;
use threadpool::ThreadPool;

#[derive(Clone, Debug, Serialize)]
struct Restaurant {
    name: String,
    address: String,
    price_range: i32,
    capacity: i32,
    table_available: i32,
}

fn main() {

    let mut restaurants : Vec<Restaurant> = vec![
        Restaurant {
            name: String::from("Bento Bistro"),
            address: String::from("1234 Main Street"),
            price_range: 2,
            capacity: 24,
            table_available: 24,
        },
        Restaurant {
            name: String::from("Ayse Pizzas"),
            address: String::from("65 Second Avenue"),
            price_range: 3,
            capacity: 13,
            table_available: 13,
        },
        Restaurant {
            name: String::from("Heavenly Havana"),
            address: String::from("5661 Soho Boulevard"),
            price_range: 4,
            capacity: 18,
            table_available: 18,
        },
        Restaurant {
            name: String::from("Kung Fu Bistro"),
            address: String::from("3300 Peterson Avenue Suite 49"),
            price_range: 3,
            capacity: 29,
            table_available: 29,
        },
        Restaurant {
            name: String::from("Only Plant Pizzas"),
            address: String::from("32A Charles Plaza"),
            price_range: 3,
            capacity: 20,
            table_available: 20,
        },
        Restaurant {
            name: String::from("Dolma and Baklava"),
            address: String::from("4559 Haziran Avenue"),
            price_range: 5,
            capacity: 20,
            table_available: 20,
        },
        Restaurant {
            name: String::from("Dotty and Tommy Cafe"),
            address: String::from("100 Fifth Avenue"),
            price_range: 2,
            capacity: 12,
            table_available: 12,
        },
        Restaurant {
            name: String::from("Seaside Bites"),
            address: String::from("44500 Oceanside Boulevard"),
            price_range: 3,
            capacity: 15,
            table_available: 15,
        },
        Restaurant {
            name: String::from("Sweet Dreams are Made Of These"),
            address: String::from("23 Sleepy Hallow Alley"),
            price_range: 2,
            capacity: 8,
            table_available: 8,
        },
        Restaurant {
            name: String::from("Enchanted"),
            address: String::from("443 Main Street 2/F"),
            price_range: 5,
            capacity: 35,
            table_available: 35,
        },
    ];

    let (req_tx, req_rx) = mpsc::channel();
    let (data_tx, data_rx) = mpsc::channel(); 

    // periodically update the capacity
    let handle = thread::spawn(move || {

        loop {

            // this loop simulates foot traffic
            for restaurant in restaurants.iter_mut() {

                let delta = (restaurant.capacity as f32 * 0.4) as i32;
                let table_change = rand::thread_rng().gen_range((delta)*-1..delta);
        
                let start = restaurant.table_available;

                restaurant.table_available += table_change;

                if restaurant.table_available > restaurant.capacity {
                    restaurant.table_available = restaurant.capacity;
                }
                else if restaurant.table_available < 0 {
                    restaurant.table_available = 0;
                }

                println!("thread: {} before {} change {} after {} (cap {})",
                 restaurant.name, start, table_change,
                 restaurant.table_available, restaurant.capacity, );
            }

            // check to see if data has been requested
            if let Ok(_) = req_rx.try_recv() {
                data_tx.send(restaurants.clone()).unwrap();
            }

            sleep(Duration::from_millis(3000));
        };

    });

    // now run the REST API server

    // TODO 1: start listening to port 4400, complete the next line
    //         end the app if unable
    let listener = TcpListener::bind("localhost:4400").expect("Unable to bind to localhost:4400");

    println!("Now listening to port 4400…");

    // TODO 4: Use a thread pool of 10 threads
    let thread_pool = ThreadPool::new(10);
    let data_mutex = Arc::new(Mutex::new(data_rx));

    for stream_result in listener.incoming() {

        let req_tx_clone = req_tx.clone();
        let data_mutex_clone = data_mutex.clone();
        if let Ok(stream) = stream_result {
            // thread_pool.execute(process_stream(stream, &req_tx, &data_rx));
            thread_pool.execute(move || {
                process_stream(stream, &req_tx_clone, data_mutex_clone);
            })
        }
    }
    
    handle.join().unwrap();

}

fn process_stream(
    mut stream: TcpStream,
    data_requester: &Sender<()>,
    data_receiver: Arc<Mutex<Receiver<Vec<Restaurant>>>>
) {
    let http_request = read_http_request(&mut stream);

    if http_request.iter().count() <= 0 {
        return;
    }

    if http_request[0].len() < 6 {
        return;
    }
    
    let test = &http_request[0][..6];
    
    if test != "GET / " {
        println!("Request {} ignored: ", http_request[0]);
        return;
    }

    let latest_restaurant_data = get_latest_restaurant_data(data_requester, data_receiver);
    dbg!(&latest_restaurant_data);
 
    send_http_response(&mut stream, &latest_restaurant_data);
}

fn read_http_request(stream: &mut TcpStream) -> Vec<String> {
    // collecting and printing the TCP request
 
    let buf_reader = BufReader::new(stream);
   
    let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap())
    .take_while(|line| !line.is_empty())
    .collect();
 
    // println!("Request: {:#?}", &http_request);
 
    return http_request;
 }

fn send_http_response(stream: &mut TcpStream, data: &Option<Vec<Restaurant>>) {

    let response_line = "HTTP/1.1 200 OK";
   
    let empty : Vec<Restaurant> = vec![];
    let data_unwrapped: &Vec<Restaurant> = 
        match data {
            None => &empty,
            Some(data) => &data,
        };

    // TODO 3: take the "data_unwrapped" from above and send it back as JSON
    //         just put it in the variable payload       

    // let payload = "<h3>This feature has not been implemented yet. JSON to be returned here.</h3>";
    let serialization_result = serde_json::to_string(data_unwrapped);
    let payload = match serialization_result {
        Ok(str) => str,
        _ => String::from("[]"),
    };

    let content_length = payload.len();
    let content_type = "application/json";
    // let content_type = "text/html";
 
    let headers = format!("Content-Length: {content_length}\r\nAccess-Control-Allow-Origin : *\r\nContent-Type: {content_type}\r\n");
 
    let http_response = format!("{response_line}\r\n{headers}\r\n{payload}");
 
    stream.write_all(http_response.as_bytes()).unwrap();
}
 
fn get_latest_restaurant_data(
    data_requester: &Sender<()>,
    data_receiver: Arc<Mutex<Receiver<Vec<Restaurant>>>>
) -> Option<Vec<Restaurant>> {

    // TODO 2 ask the simulation thread for data
    //        set the time out to 8000 milliseconds
    //        remove the next line, it is just here so that the code compiles
    data_requester.send(()).unwrap();
    match data_receiver.lock().unwrap().recv_timeout(Duration::from_millis(8000)) {
        Ok(data) => Some(data),
        _ => None,
    }
    // return None;
 
}
