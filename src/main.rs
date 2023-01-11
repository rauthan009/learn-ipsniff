use std::{
    env,
    io::{self, Write},
    net::{IpAddr, TcpStream},
    process,
    str::FromStr,
    sync::mpsc::{channel, Sender},
    thread,
};

const MAX: u16 = 65535;

#[derive(Debug)]
struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough argument");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }
        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            Ok(Arguments {
                flag: String::from(""),
                ipaddr,
                threads: 5,
            })
        } else {
            let flag = args[1].clone();
            if (flag.contains("-h") || flag.contains("-help")) && args.len() == 3 {
                println!(
                    "Usage: -j to select how many threads you want
                    \r\n -h or -help to show this help message"
                );
                Err("help")
            } else if flag.contains("-h") || flag.contains("-help") {
                Err("too few arguments")
            } else if flag.contains("-j") {
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a valid IPADDR. must be IPV5 or IPV6"),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("Failed to parse thread number"),
                };

                Ok(Arguments {
                    flag,
                    ipaddr,
                    threads,
                })
            } else {
                Err("invalid syntax")
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port = start_port + 1;
    loop {
        if TcpStream::connect((addr, port)).is_ok() {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(port).unwrap();
        }

        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(0);
        }
    });
    println!("flag: {} ,arguments {:?}", arguments.flag, arguments);
    let num_threads = arguments.threads;
    let addr = arguments.ipaddr;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
