use std::thread;
use std::time::Duration;
use std::thread::sleep;
use std::sync::mpsc::{self};

use rand::Rng;

const FULL_TANK: i32 = 5000;
const MAX_CONSUMPTION : i32 = 300;

fn main() {

    let mut rocket_fuel_tank = FULL_TANK;

    // TODO: Uncomment the next two lines and use these for inter-thread communications
    let (rocket_fuel_req_tx, rocket_fuel_req_rx) = mpsc::channel();
    let (rocket_fuel_data_tx, rocket_fuel_data_rx) = mpsc::channel(); 

    // spawn a thread to simulate fuel consumption

    let handle = thread::spawn(move || {

        loop {

            let consumption = rand::thread_rng().gen_range(1..MAX_CONSUMPTION);

            let starting = rocket_fuel_tank;

            rocket_fuel_tank -= consumption;
            if rocket_fuel_tank <= 0 { rocket_fuel_tank = 0; }

            println!("spawned thread: {starting} - {consumption} = {rocket_fuel_tank} remaining");

            // TODO: check to see if data has been requested
            //       send it if it has been requested
            if let Ok(_) = rocket_fuel_req_rx.try_recv() {
                rocket_fuel_data_tx.send(rocket_fuel_tank.clone()).unwrap();
            }

            if rocket_fuel_tank == 0 {
                println!("spawned thread ended.");
                break;
            }
            else {
                sleep(Duration::from_millis(100));
            }
        };

    });

    // now poll the remaining volume until the tank is empty

    loop {
        
        if rocket_fuel_req_tx.send(()).is_ok() {
            // TODO: write the code for the if block (replacing false)
            //       to request data from the spawned thread here
            
            // println!("Rocket fuel remaining: {data}"),
            // println!("No response from spawned thread. Assuming 0.")
            match rocket_fuel_data_rx.recv_timeout(Duration::from_millis(5000)) {
                Ok(data) => println!("Rocket fuel remaining: {data}"),
                _ => println!("No response from spawned thread. Assuming 0.")
            }
        }
        else {
            println!("Could not request data from spawned thread. Assuming 0.");
            break;
        }
    }
    
    handle.join().unwrap();

}