use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Error};
use std::sync::{Arc, Mutex};

use clap::{App, Arg};
use colored::*;
use log::{info, warn, error, trace, Level, LevelFilter, Metadata, Record};

mod constants;

struct SocatRsLogger;

impl log::Log for SocatRsLogger
{
    fn enabled(&self, metadata: &Metadata) -> bool
    {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record)
    {
        if self.enabled(record.metadata())
        {
            let level = match record.level().to_string().as_str()
            {
                "ERROR" => { "ERROR".red() },
                "WARN" => { "WARN".magenta() },
                "INFO" => { "INFO".green() },
                "DEBUG" => { "DEBUG".cyan() },
                _ => { "TRACE".bold() },
            };

            println!("[{}] - {}", level, record.args());
        }
    }

    fn flush(&self) {}
}


static LOGGER: SocatRsLogger = SocatRsLogger;


struct ThreadParameters
{
    target: String
}


impl ThreadParameters
{
    fn new(t: String) -> Self
    {
        ThreadParameters
        {
            target: t.to_string(),
        }
    }
}



fn handle_tcp_client(mut to_client_stream: TcpStream, args: Arc<Mutex<ThreadParameters>>) -> Result<(), Error>
{
    trace!("handling connection from {}", to_client_stream.peer_addr()?);


    let dst = &args.lock().unwrap().target;
    let mut to_server_stream = TcpStream::connect(dst).expect("failed to connect to server");

    // not async at all, todo
    loop
    {
        let mut buf = [0; 2048];
        // cli -> srv
        let request_nb_bytes = to_client_stream.read(&mut buf)?;
        to_server_stream.write(&buf[..request_nb_bytes])?;

        // srv -> cli
        let response_nb_bytes = to_server_stream.read(&mut buf)?;
        to_client_stream.write(&buf[..response_nb_bytes])?;
    }
}


fn tcp_forward(source: String, destination: String) -> Result<(), Error>
{
    let srv = TcpListener::bind(source)
        .expect("failed to bind port");

    let params = ThreadParameters::new(destination);
    let rc = Arc::new(Mutex::new(params));

    for conn in srv.incoming()
    {
        match conn
        {
            Err(e) =>
            {
                println!("connection failed, reason: {}", e)
            }

            Ok(conn) =>
            {
                let local_rc = Arc::clone(&rc);

                let t = std::thread::spawn(move ||
                {
                    handle_tcp_client(conn, local_rc)
                        .unwrap_or_else(|err| eprintln!("{:?}", err))
                });

                t.join().unwrap();
            }
        }
    }

    Ok(())
}


fn main()
{
    let tcp_app = App::new("tcp")
        .arg(
            Arg::with_name("source")
            .short('s')
            .long("source")
            .about("TCP source (format as PORT or IP:PORT or FQDN:PORT)")
            .takes_value(true)
            .required(true)
        )

        .arg(
            Arg::with_name("destination")
            .short('d')
            .long("destination")
            .about("TCP destination (format as PORT or IP:PORT or FQDN:PORT)")
            .takes_value(true)
            .required(true)
        )
    ;

    let udp_app = App::new("udp");

    let app = App::new(constants::APPLICATION_SHORTNAME)
        .version(constants::VERSION)
        .author(constants::AUTHOR)
        .about(constants::DESCRIPTION)

        .arg(
            Arg::with_name("verbosity")
                .short('v')
                .about("Increase verbosity (repeatable from 1 - info - to 4 - debug)")
                .multiple(true)
                .takes_value(false)
        )

        .subcommand( tcp_app )
        .subcommand( udp_app )
    ;


    let matches = app.get_matches();

    let verbosity = match matches.values_of("verbosity")
    {
        Some(_x) =>
        {
            let cnt = matches.occurrences_of("verbosity");
            trace!("setting verbosity to {}", cnt);
            match cnt
            {
                4 => { LevelFilter::Trace } // -vvvv
                3 => { LevelFilter::Debug } // -vvv
                2 => { LevelFilter::Info }  // -vv
                1 => { LevelFilter::Warn }  // -v
                _ => { LevelFilter::Error }
            }
        },
        None => { LevelFilter::Error }
    };

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(verbosity))
        .unwrap();



    match matches.subcommand_name() {
        Some("tcp") =>
        {
            trace!("tcp_forwarding");

            if let Some(ref submatches) = matches.subcommand_matches("tcp")
            {
                let source = match submatches.value_of("source")
                {
                    Some(x) =>
                    {
                        x.to_string()
                    },
                    None => panic!("argument source is required")
                };

                let destination = match submatches.value_of("destination")
                {
                    Some(x) =>
                    {
                        x.to_string()
                    },
                    None => panic!("argument source is required")
                };

                tcp_forward(source, destination)
                    .unwrap_or_else(|e| error!("failed to forward: {:?}", e));
            }
        },

        Some("udp") =>
        {
            trace!("udp_forwarding");
            todo!("")
        },

        None => warn!("No subcommand was used"),
        _ => error!("Some other subcommand was used"),
    }
}