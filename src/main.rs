use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write, Error};


fn handle_client(mut to_client_stream: TcpStream) -> Result<(), Error>
{
    println!("handling connection from {}", to_client_stream.peer_addr()?);
    let mut to_server_stream = TcpStream::connect("127.0.0.1:5557").expect("failed to connect to server");

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



fn main()
{
    let srv = TcpListener::bind("0.0.0.0:27002").expect("failed to bind port");
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
                thread::spawn(move || {
                    handle_client(conn)
                        .unwrap_or_else(|err| eprintln!("{:?}", err));
                });
            }
        }
    }
}
