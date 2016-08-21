extern crate ws;
extern crate rustc_serialize;

const BOARD_WIDTH: usize = 7;

enum Command {
    PlacePiece { position: AxialPoint },
}

struct AxialPoint {
    q: isize,
    r: isize,
}

#[derive(Copy,Clone)]
enum CellState {
    Red,
    Blue,
    Empty,
}

struct GameState {
    grid: [CellState; BOARD_WIDTH * BOARD_WIDTH],
}

impl GameState {
    fn is_in_bounds(point: &AxialPoint) -> bool {
        let radius = (BOARD_WIDTH / 2) as isize;
        (point.q >= -radius && point.q <= radius &&
            point.r >= -radius && point.r <= radius)
    }

    fn flatten_point(point: &AxialPoint) -> Result<usize, ()> {
        if GameState::is_in_bounds(point) {
            let radius = (BOARD_WIDTH / 2) as isize;
            let q_offset = (point.q + radius) as usize;
            let r_offset = (point.r + radius) as usize;
            Ok(q_offset + r_offset * BOARD_WIDTH)
        } else {
            Err(())
        }
    }

    fn get(&self, point: &AxialPoint) -> Result<CellState, ()> {
        let flattened = try!(GameState::flatten_point(point));
        Ok(self.grid[flattened])
    }

    fn get_mut(&mut self, point: &AxialPoint) -> Result<&mut CellState, ()>{
        let flattened = try!(GameState::flatten_point(point));
        Ok(&mut self.grid[flattened])
    }
}

struct Server {
    sender: ws::Sender,
    game: GameState,
}

impl Server {
    fn message_to_command(message: ws::Message) -> Result<Command, (ws::CloseCode, String)> {
        let text = match message {
            ws::Message::Text(text) => text,
            _ => return Err((ws::CloseCode::Unsupported, "Only text allowed".to_string())),
        };

        let json = rustc_serialize::json::Json::from_str(&text).unwrap();
        let obj = json.as_object().unwrap();

        Err((ws::CloseCode::Error, "Butts".to_string()))
    }
}

impl ws::Handler for Server {
    fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
        if let ws::Message::Text(text) = message {
            self.sender.send(text)
        } else {
            self.sender.close_with_reason(ws::CloseCode::Unsupported, "Only text data is allowed")
        }
    }

    fn on_open(&mut self, handshake: ws::Handshake) -> ws::Result<()> {
        if let Ok(Some(ip)) = handshake.remote_addr() {
            println!("Opening connection to {}", ip);
        } else {
            println!("Opening connection to [unknown]");
        }

        Ok(()) 
    }

    fn on_close(&mut self, _: ws::CloseCode, _: &str) {
        println!("Closing connection");
    }
}

fn main() {
    ws::listen("127.0.0.1:3012", |sender| Server {
        sender: sender,
        game: GameState { grid: [CellState::Empty; BOARD_WIDTH * BOARD_WIDTH] }
    } ).unwrap()
}
