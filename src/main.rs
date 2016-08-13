extern crate mio;

use std::net::SocketAddr;

use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use std::collections::hash_map::HashMap;

const SERVER: Token = Token(0);

struct GameServer {
    server_socket: TcpListener,

    // Store our connected clients by token
    clients: HashMap<Token, TcpStream>,

    // The number of client tokens ever created. We use this to create unique
    // tokens when clients connect to us.
    token_counter: usize,
}

impl GameServer {
    /**
     * Create a new game server and register it with the event loop.
     *
     * This is a convenience function that binds to the given address and
     * starts listening on it.
     */
    fn create(event_loop: &mut EventLoop<GameServer>, addr: &SocketAddr) -> GameServer {
        // Create our server socket and tell the event loop we're listening on
        // it.
        let server_socket = TcpListener::bind(&addr).unwrap();
        event_loop.register(&server_socket, SERVER, EventSet::readable(),
                            PollOpt::edge()).unwrap();  

        GameServer { server_socket: server_socket, token_counter: 0, clients: HashMap::new() }
    }

    /**
     * Create a new, unused client token.
     */
    fn create_client_token(&mut self) -> Token {
        self.token_counter += 1;
        assert!(Token(self.token_counter) != SERVER);

        Token(self.token_counter)
    }
}

impl Handler for GameServer {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<GameServer>, token: Token, _: EventSet) {
        match token {
            SERVER => {
                let (client_socket, client_addr) = match self.server_socket.accept() {
                    Err(e) => {
                        println!("Could not accept client connection, got error: {:?}", e);
                        return;
                    },
                    Ok(None) => unreachable!(),
                    Ok(Some(client)) => client,
                };

                let client_token = self.create_client_token();

                // Add the socket to our client hash, and panic if there's a collision
                let insert_result = self.clients.insert(client_token, client_socket);
                assert!(insert_result.is_none(),
                        "Client token collision at {:?}", client_token);

                // Start listening on the new client socket
                event_loop.register(&self.clients[&client_token], client_token,
                                    EventSet::readable(), PollOpt::edge()).unwrap();

                println!("Client connected from {:?}", client_addr);
            },
            client_token => {
                let client_socket = self.clients.get_mut(&client_token).unwrap();

                use std::io::Read;
                let mut buffer: Vec<u8> = vec![];
                match client_socket.read_to_end(&mut buffer) {
                    Ok(num_bytes) => println!("Read {} bytes from {:?}", num_bytes, client_socket),
                    Err(error) => println!("Got error reading from {:?}: {:?}",
                                           client_socket.peer_addr().unwrap(),
                                           error),
                }

                use std::str;
                println!("Recieved text: {}", str::from_utf8(&buffer).unwrap());
            },
        }
    }
}

fn main() {
    let addr = "127.0.0.1:13265".parse().unwrap();

    // Create an event loop
    let mut event_loop = EventLoop::<GameServer>::new().unwrap();  

    // Start handling events
    let mut game_server = GameServer::create(&mut event_loop, &addr);
    event_loop.run(&mut game_server).unwrap();
}
