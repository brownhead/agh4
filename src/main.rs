extern crate mio;

use std::net::SocketAddr;

use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use std::collections::hash_map::HashMap;

enum PeerState {
    // The initial state when we're waiting for the peer to upgrade their HTTP
    // connection to the websocket protocol.
    AwaitingWebSocketUpgrade,

    // The peer is communicating through the websocket protocol now.
    Upgraded,
}

struct ConnectedPeer {
    socket: TcpStream,
    buffer: Vec<u8>,
    state: PeerState,
}

const SERVER: Token = Token(0);

struct GameServer {
    server_socket: TcpListener,

    // Store our connected peers by token
    peers: HashMap<Token, ConnectedPeer>,

    // The number of peer tokens ever created. We use this to create unique
    // tokens when peers connect to us.
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

        GameServer { server_socket: server_socket, token_counter: 0, peers: HashMap::new() }
    }

    /**
     * Create a new, unused peer token.
     */
    fn create_peer_token(&mut self) -> Token {
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
                let (peer_socket, peer_addr) = match self.server_socket.accept() {
                    Err(e) => {
                        println!("Could not accept connection, got error: {:?}", e);
                        return;
                    },
                    Ok(None) => unreachable!(),
                    Ok(Some(peer)) => peer,
                };

                let peer_token = self.create_peer_token();

                // Add the socket to our peer hash, and panic if there's a collision
                let insert_result = self.peers.insert(
                    peer_token,
                    ConnectedPeer {
                        socket: peer_socket,
                        buffer: vec![],
                        state: PeerState::AwaitingWebSocketUpgrade,
                    });
                assert!(insert_result.is_none(),
                        "peer token collision at {:?}", peer_token);

                // Start listening on the new peer socket
                event_loop.register(&self.peers[&peer_token].socket, peer_token,
                                    EventSet::readable(), PollOpt::edge()).unwrap();

                println!("peer connected from {:?}", peer_addr);
            },
            peer_token => {
                let ref mut peer_socket = self.peers.get_mut(&peer_token).unwrap().socket;

                use std::io::Read;
                let mut buffer: Vec<u8> = vec![];
                match peer_socket.read_to_end(&mut buffer) {
                    Ok(num_bytes) => println!("Read {} bytes from {:?}", num_bytes, peer_socket),
                    Err(error) => println!("Got error reading from {:?}: {:?}",
                                           peer_socket.peer_addr().unwrap(),
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
