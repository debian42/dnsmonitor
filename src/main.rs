use dnsmonitor::{make_dns_call, metric_thread, Metrics};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex,atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use std::{env, thread};

fn find_help_flag(flags: &mut [String]) -> bool {
    let found = flags.iter().position(|x| x == "-h");
    match found {
        Some(_pos) => {
            true
         },
        None => false
    }
}

fn find_and_remove_debug_flag(flags: &mut Vec<String>) -> bool
{
    let found = flags.iter().position(|x| x == "-d");
    match found {
        Some(pos) => {
            flags.remove(pos);
            true
         },
        None => false
    }
}

fn find_and_remove_sleep_value_in_ms(flags: &mut Vec<String>) -> u64
{
    const DEFAULT: u64 = 1000;
    let found = flags.iter().position(|x| x == "-s");
    match found {
        Some(pos) => {
            flags.remove(pos);
            let value = flags.get(pos);
            if let Some(wait_time) = value {
                let int_value = wait_time.parse::<u64>();
                if int_value.is_err() {
                    println!("\x1b[91mwait time not a number\x1b[0m");
                    std::process::exit(1);
                }
                flags.remove(pos);
                int_value.unwrap()
            } else {
                println!("\x1b[91mno wait time given\x1b[0m");
                std::process::exit(2);
            }
         },
        None => DEFAULT
    }
}

fn find_and_remove_warning_time_in_ms(flags: &mut Vec<String>) -> u32
{
    const DEFAULT: u32 = 5; // after 5ms we print a waring to the console
    let found = flags.iter().position(|x| x == "-w");
    match found {
        Some(pos) => {
            flags.remove(pos);
            let value = flags.get(pos);
            if let Some(wait_time) = value {
                let int_value = wait_time.parse::<u32>();
                if int_value.is_err() {
                    println!("\x1b[91mwarning time not a number\x1b[0m");
                    std::process::exit(1);
                }
                flags.remove(pos);
                int_value.unwrap()
            } else {
                println!("\x1b[91mno warning time given\x1b[0m");
                std::process::exit(2);
            }
         },
        None => DEFAULT
    }
}

// cargo run --release -- -s 100 -w 1000 -d www.heise.de www.ka.ka
fn main() 
{
    // I want my lovely c++ back ;-(
    // TODO: Check if we have really no deadlock 
    let dns_failures = Arc::new(RwLock::new(HashMap::new()));

    // We expect everything as an dns name
    let mut dns_names: Vec<String> = env::args().collect();
    
    if find_help_flag(&mut dns_names) {
        println!("\x1b[96mNo help possible!\x1b[0m");
        return;
    }

    // uncomment for quick debugging
    //-s 1001 -w 1 -d www.heise.de ldap-db-prod.telekom.tv SOABP-PRD.de.t-internal.ai
    /*
        dns_names.push("-s".to_owned());
        dns_names.push("1001".to_owned());
        dns_names.push("-d".to_owned());
        dns_names.push("www.heise.de".to_owned());
        dns_names.push("ldap-db-prod.telekom.tv".to_owned());
        dns_names.push("SOABP-PRD.de.t-internal.ai".to_owned());
    */
    let debug = find_and_remove_debug_flag(&mut dns_names);
    
    if debug {
        println!("{} arguments passed: {:?} also debug enabled(-d) ", dns_names.len(), dns_names);
    }

    let wait_time = find_and_remove_sleep_value_in_ms(&mut dns_names);
    let warning_time = find_and_remove_warning_time_in_ms(&mut dns_names);

    if dns_names.len() == 1 {
        println!(
            "\x1b[91mNo arguments given.\x1b[0m\n\x1b[94m E.g.: {} www.heise.de. www.ibm.com.\x1b[0m",
            dns_names.first().unwrap()
        );
    } else {
        dns_names.remove(0); // remove our self

        // A vector containing all the JoinHandles for the spawned threads
        let mut fetch_handles: Vec<thread::JoinHandle<()>> = Vec::new();

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
            println!("Terminating ...");
        })
        .expect("Error setting Ctrl-C handler");

        let r2 = running.clone();
        let dns_failures_clone = dns_failures.clone();

        // start_metric_thread
        let handle =  thread::Builder::new().name("metric_thread".to_string())
        .spawn(move || {
            metric_thread(8080, r2, &dns_failures, debug);
        });
        fetch_handles.push(handle.expect("Could not start metric thread"));

        
        // insert hosts in hashmap
        for i in 0..dns_names.len()
        {
            let mut map = dns_failures_clone.write().expect("RwLock failed ?!?!");
            let hostname = dns_names.get(i).unwrap().clone();
            map.entry(hostname).or_insert_with(|| Mutex::new(Metrics::new()));
        }

        // create one thread for each host
        for i in 0..dns_names.len() 
        {
            let hostname = dns_names.get(i).unwrap().clone();
            let r = running.clone();
            let dns_failures = Arc::clone(&dns_failures_clone);
            let handle =  thread::Builder::new().name(hostname.clone())
            .spawn(move || {
                let mut counter: u64 = 0;
                loop {
                    if !r.load(Ordering::SeqCst) {
                        break;
                    }
                    counter += 1;
                    let (duration, ip_address) = make_dns_call(&hostname, counter, &dns_failures, debug);
                    let duration_ms:f64 = duration as f64 / 1000.0;
                    if duration_ms > warning_time as f64 {
                        // {:-<40}  : Warning: ldap-db-tkom.tv----------------- took:
                        println!("\x1b[93mWarning: {: <40}{:17} took: \x1b[94m{:6} µs  \x1b[93mcalled: {:7}\x1b[0m", &hostname, ip_address, duration, counter);
                    }
                    Metrics::average(&dns_failures, &hostname, duration);
                    let remaining_sleep_time:i64 = wait_time as i64 - duration_ms as i64 ;
                    if debug { 
                        println!("remaining_sleep_time: {} ms", remaining_sleep_time);
                    }
                    if remaining_sleep_time > 0 {
                        thread::sleep(Duration::from_millis(remaining_sleep_time as u64));
                    }
                }
            });
            fetch_handles.push(handle.expect("Could not start dns thread"));
        }

        // join all the threads
        while !fetch_handles.is_empty() 
        {
            let cur_thread = fetch_handles.remove(0);
            cur_thread.join().unwrap();
        }

        // print stats        
        let map = dns_failures_clone.read().expect("RwLock failed ?!?!");
        for (key, value) in &*map {
            println!("{} / {:?}", key, value.lock().unwrap());
        }        
    }
}
