use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use integer_encoding::{VarIntAsyncReader, VarIntReader, VarIntWriter};

// craft packets

struct Packet {
    id: u32, // ID/packet type or whatever
    data: Vec<u8> // u8 used for varints
}

// use u32 becaus apparently claude said integer_encoding uses zigzag encodign on i32, and already
// our server only uses positive values or whatever yeah meheh

// Geniunely wnat the fuck is this i'm sleepy as shit
async fn read_packet(stream: &mut TcpStream) -> std::io::Result<(u32, Vec<u8>)> {
    let packet_length: u32 = stream.read_varint_async().await?; // Get packet length
    let mut server_packet_data = vec![0u8; packet_length as usize]; // Create vec w/ packet
                                                                             // length as size
    stream.read_exact(&mut server_packet_data).await?; // and read the exact length of bytes provided by
                                                   // server!!
    // here we parse the packet

    let mut cur = &server_packet_data[..]; // take the um data
    let id: u32 = cur.read_varint()?; // read packet id
    Ok((id, cur.to_vec()))
}

impl Packet {
    fn new(id: u32) -> Self {
        Packet {id, data: Vec::new()}
    }
    

    fn write_string(&mut self, s: &str) {
        self.data.write_varint(s.len() as u32); // add the len, because minecraft usually takes
                                                // strings as like len then the string and whatever
        self.data.extend_from_slice(s.as_bytes());
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // create a vec w/ the id
        let mut packet_data: Vec<u8> = Vec::new();
        packet_data.write_varint(self.id as u32).unwrap();
        packet_data.extend_from_slice(&self.data);

        // now create the full packet
        let mut full_packet: Vec<u8> = Vec::new();
        full_packet.write_varint(packet_data.len() as u32).unwrap();
        full_packet.extend_from_slice(&packet_data);
        full_packet
    }
    
    async fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let full_packet = self.to_bytes();
        stream.write_all(&full_packet).await?;
        Ok(())
    }
    // easily can do with self.data.write_varint(int);
    //fn varint_integer(&mut self, int: u32) {
    //    self.data.write_varint(int)
    //}
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("minecraft join and then wait and then just fucking fermer le connection!!!!");
    println!("Connecting to server on 127.0.0.1:25565...");
    let mut stream = TcpStream::connect("127.0.0.1:25565").await?;
    println!("Initiating protocol handshake..");
    let mut pk = Packet::new(0x00);
    //
    // Handshake in Minecraft Java (0x00)
    //
    // VarInt: Protocol Version (777 for 26.3)
    // String (write_string): Server Address
    // Unsigned Short (u16 to_be_bytes): Server Port
    // VarInt: Intent / 1 for Status (response is JSON) - 2 for Login - 3 for Transfer
    // static bc i dont wanna make it easy YET but ig port is here
    let port: u16 = 25565;
    pk.data.write_varint(777u32).unwrap(); // Protocol version 777 for 26.3
    pk.write_string("127.0.0.1"); // Hardcoded server address
    pk.data.extend_from_slice(&port.to_be_bytes()); // Port 25565 :3
    pk.data.write_varint(1u32).unwrap(); // 1 for Status
    pk.send(&mut stream).await?;
    println!("Handshake sent!");
    println!("Sending status request and retrieving server JSON..");
    //
    // Status Request
    // No fields, just craft a basic packet w/ 0x00 and youre done. You can send ping + pong but not
    // needed for a beginner project
    let mut pk = Packet::new(0x00);
    pk.send(&mut stream).await?;
    let (id, fbody) = read_packet(&mut stream).await?;
    let mut cur = &fbody[..];
    let json_len: u32 = cur.read_varint()?;
    let json = String::from_utf8_lossy(&cur[..json_len as usize]);
    println!("{id}: {json}");
    stream.shutdown().await?;
    Ok(())
}
