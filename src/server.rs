use std::{sync::mpsc::{Sender, self, Receiver}, thread::{JoinHandle, spawn}, net::{SocketAddr, IpAddr, TcpListener, TcpStream}};



use crate::{custom_errors::NodeCustomErrors, node::Node};

const LOCALHOST: &str = "127.0.0.1";

enum NodeServerMessage {
    Finsih,
}

pub struct NodeServer {
    sender: Sender<NodeServerMessage>,
    handle: JoinHandle<Result<(), NodeCustomErrors>>,
}

impl NodeServer {
    pub fn new(port: u16) -> NodeServer {
        let (sender, rx) = mpsc::channel();
        let address = get_socket(LOCALHOST.to_string(), port);
        let handle = spawn(move || {
            Self::listen(address, rx)
        });
        NodeServer { sender, handle}
    }

    fn listen(address: SocketAddr, rx: Receiver<NodeServerMessage>) -> Result<(), NodeCustomErrors> {
        let address = format!("{}:{}", address.ip(), address.port());
        let listener: TcpListener = TcpListener::bind(&address).unwrap();
        listener.set_nonblocking(true).unwrap();
        for stream in listener.incoming() {
            // recivio un mensaje para frenar
            if rx.try_recv().is_ok() {
                break;
            }
            match stream {
                Ok(stream) => {
                    Self::handle_incoming_connection(stream)?;
                } 
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    // This doesen't mean an error ocurred, there just wasn't a connection at the moment
                    println!("ERROR DE WOULDBLOCK");
                }
                Err(err) => return Err(NodeCustomErrors::CanNotRead(err.to_string())),
            }
        }
        Ok(())
    }


    fn handle_incoming_connection(stream: TcpStream) -> Result<(), NodeCustomErrors> {

        Ok(())
    }


    pub fn finish(self) -> Result<(), NodeCustomErrors> {
        let _ = self.sender.send(NodeServerMessage::Finsih);
        self.handle.join().map_err(|_| NodeCustomErrors::ThreadJoinError("Error al hacer join al thread del servidor que esucha".to_string()))??;
        Ok(())
    }
}


fn get_socket(ip: String, port: u16) -> SocketAddr {
    let ip: IpAddr = ip.parse::<IpAddr>().unwrap();
    SocketAddr::new(ip, port)
}