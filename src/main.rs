use clap::{App, Arg};
use colored::*;
use log::{warn, error, trace, Level, LevelFilter, Metadata, Record};

mod constants;
mod io;

use crate::io::tcp::tcp_forward;


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


pub struct ThreadParameters
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



    match matches.subcommand_name()
    {
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