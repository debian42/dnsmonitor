use std::net::{ToSocketAddrs, TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use std::time::Duration;
use std::{
    thread,
    io,
    io::{prelude::*, BufReader},
    collections::HashMap,
};


#[derive(Debug)]
pub struct Metrics {
    // in microseconds
    pub sum:u64,
    pub average_count:u64,
    pub min:u64,
    pub max:u64,
    pub failure_count:u64,
}

impl Metrics {
    // FIXME: These functions doesn't really work on the "object", but I wanted to test it: make it free features
    fn set_min_max_and_increment_counter(dns_failures: &Arc<RwLock<HashMap<String, Mutex<Metrics>>>>, dns_name: &str, took: u64) {
        let map = dns_failures.write().expect("RwLock poisoned");
        if let Some(element) = map.get(dns_name)
        {
            let mut element = element.lock().expect("Mutex failed !?!?");                
            let metrics = &mut *element;
            metrics.failure_count += 1;
            if took < metrics.min {
                metrics.min = took;
            }
            if took > metrics.max {
                metrics.max = took;
            }
        }    
    }
    fn set_min_max(dns_failures: &Arc<RwLock<HashMap<String, Mutex<Metrics>>>>, dns_name: &str, took: u64) {
        let map = dns_failures.write().expect("RwLock poisoned");
        if let Some(element) = map.get(dns_name)
        {
            let mut element = element.lock().expect("Mutex failed !?!?");                
            let metrics = &mut *element;            
            if took < metrics.min {
                metrics.min = took;
            }
            if took > metrics.max {
                metrics.max = took;
            }
        }    
    }

    pub fn clear(&mut self)
    {
        self.sum = 0;
        self.min = u64::MAX;
        self.max = 0;
        self.failure_count = 0;
        self.average_count = 0;
    }

    pub fn average(dns_failures: &Arc<RwLock<HashMap<String, Mutex<Metrics>>>>, dns_name: &str, duration: u64) {
        let map = dns_failures.write().expect("RwLock poisoned");
        if let Some(element) = map.get(dns_name)
        {
            let mut element = element.lock().expect("Mutex failed !?!?");                
            let metrics = &mut *element;
            metrics.sum += duration;
            metrics.average_count += 1;
        //    println!("{}:   {}  /  {}  =  {}", duration, metrics.sum, metrics.average_count, metrics.sum as f64 / metrics.average_count as f64);
        }    
    }

    pub fn new() -> Metrics {
        Metrics { sum: 0, min: u64::MAX, max: 0, failure_count: 0, average_count: 0}
    }
}

// Use the system's DNS resolver
pub fn make_dns_call(dns_name: &str, counter: u64, dns_failures: &Arc<RwLock<HashMap<String, Mutex<Metrics>>>>, debug_output: bool) -> (u64, String)
{
    let name = dns_name.to_owned() + ":42";
    let now = SystemTime::now();
    let address = name.to_socket_addrs();
    let dns = match address {
        Ok(mut iter) => iter.next(),
        Err(_r) => {
           // if debug_output {
           //     println!("{}: {:?}", dns_name, r);
           // } 
            None
        }
    };    
    let duration = now.elapsed().expect("Clock drifted").as_micros() as u64;

    match dns 
    {
        Some(ip) => {
            let ip_address_port = &ip.to_string();
            let ip_address: &str = &ip_address_port[0..ip_address_port.len() - 3];            
            
            if debug_output {
                println!("DNS: {: <40}{:17} took: {:6} µs  called: {:7}", dns_name, ip_address, duration, counter)
            }

            Metrics::set_min_max(dns_failures, dns_name, duration);
            (duration, ip_address.to_string())
        },
        None => {
            // ansi color octal : \033[31;1;4m   red
            // rust: \x1b[91m TEXT \x1b[0m       red
            println!("DNS: {: <40}{:26} took: {:6} µs  called: {:7}", dns_name, "\x1b[91mNOT_FOUND\x1b[0m", duration, counter);

            Metrics::set_min_max_and_increment_counter(dns_failures, dns_name, duration);
            (duration, "NOT_FOUND".to_string())
        }
    }
}


/*
# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_counter Custom metric returning the failures per dns name.
# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_counter counter
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_counter{dns="www.heise.de"} 4
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_counter{dns="www.ibm.de"} 14
 */
fn handle_http_connection(mut stream: &mut TcpStream, dns_failures: &Arc<RwLock<HashMap<String, Mutex<Metrics>>>>, debug: bool) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next(); //.unwrap().unwrap();
    stream.set_nonblocking(false).expect("Cannot set blocking");
    let request_line = match request_line {
        Some(x) => {
            match x {
                Ok(x) => x,
                Err(_err ) => "".to_owned()
            }
        }
        None => {"".to_owned()}
    };

    let status_line = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/plain";
    if debug { 
        println!("{:?}", request_line);
    }
    if request_line == "GET /metrics HTTP/1.1" 
    {
        // Scope for releasing the locks....
        {
            // FAILURE
            const HELP_FAILURE: & str = "# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter Custom metric returning the failures per dns name.\n";
            const TYPE_FAILURE: & str = "# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter gauge\n";        
            let mut contents: String = HELP_FAILURE.to_owned();
            contents.push_str(TYPE_FAILURE);

            let map = dns_failures.read().expect("RwLock failed ?!?!");
            for (key, value) in &*map 
            {
                let guard = value.lock().unwrap();
                let metrics = &*guard;

                let prom_metric = format!("yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter{{dns=\"{}\"}} {}\n", key, metrics.failure_count);
                contents.push_str(&prom_metric);
            }
            
            let map = dns_failures.read().expect("RwLock failed ?!?!");
            for (key, value) in &*map 
            {
                let guard = value.lock().unwrap();
                let metrics = &*guard;
                if metrics.min != u64::MAX 
                {
                    const HELP_MIN: & str = "# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min Custom metric returning the min time in microseconds per dns name.\n";
                    const TYPE_MIN: & str = "# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min gauge\n";        
                    contents.push_str(HELP_MIN);
                    contents.push_str(TYPE_MIN);
                    let prom_metric = format!("yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min{{dns=\"{}\"}} {}\n", key, metrics.min);
                    contents.push_str(&prom_metric);
                };
                
            }
            
            // MAX
            const HELP_MAX: & str = "# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max Custom metric returning the max time in microseconds per dns name.\n";
            const TYPE_MAX: & str = "# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max gauge\n";
            contents.push_str(HELP_MAX);
            contents.push_str(TYPE_MAX);
            let map = dns_failures.read().expect("RwLock failed ?!?!");
            for (key, value) in &*map 
            {
                let guard = value.lock().unwrap();
                let metrics = &*guard;

                let prom_metric = format!("yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max{{dns=\"{}\"}} {}\n", key, metrics.max);
                contents.push_str(&prom_metric);
            }

            // AVERAGE
            let map = dns_failures.read().expect("RwLock failed ?!?!");
            for (key, value) in &*map 
            {
                let guard = value.lock().unwrap();
                let metrics = &*guard;
                let average_value = metrics.sum as f64 / metrics.average_count as f64;
                if !average_value.is_nan() 
                {
                    const HELP_AVERAGE: & str = "# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average Custom metric returning the average time in microseconds per dns name.\n";
                    const TYPE_AVERAGE: & str = "# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average gauge\n";
                    contents.push_str(HELP_AVERAGE);
                    contents.push_str(TYPE_AVERAGE);
                    let prom_metric = format!("yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average{{dns=\"{}\"}} {}\n", key, average_value);
                    contents.push_str(&prom_metric);
                }
            }

            if debug { 
                println!("{}", contents);
            }

            let length = contents.len();
            let response = format!(
                "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        // Clear metrics
        if debug {
            println!("Clear metrics ...");
        }
        let map = dns_failures.write().expect("RwLock failed ?!?!");
        for value in (*map).values()
        {
            let mut guard = value.lock().unwrap();
            let metrics = &mut *guard;
            metrics.clear();
        }
    } 
    else
    {
        // not a call to /metrics        
        let response = "There's nothing to see here.";
        let length = response.len();
        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{response}"
        );
        if debug {
            println!("Not Metrics. Sending: {}", &response);
        }
        stream.write_all(response.as_bytes()).unwrap();
    }
}

pub fn metric_thread(port: u16, running: Arc<AtomicBool>, dns_failures: &Arc<RwLock<HashMap<String, Mutex<Metrics>>>>, debug: bool)
{
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("Could not listen");
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    for stream in listener.incoming() 
    {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        match stream {
            Ok(mut s) => {
                // send metrics, if rigth uri
                handle_http_connection(&mut s, dns_failures, debug);
                s.shutdown(Shutdown::Both).expect("Could not close socket");
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // FIXME: use select/epoll with timeout
                // Don't consume so much cpu. Only to react on ctrl+c/signals
                thread::sleep(Duration::from_millis(23));
                continue;
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }
}

