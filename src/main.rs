use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use integer_encoding::{VarIntAsyncReader, VarIntReader, VarIntWriter};
use flate2::read::ZlibDecoder;
// use std::iter::repeat;
use std::sync::atomic::{AtomicI32, Ordering};
use std::io::Read;

static compression_threshold: AtomicI32 = AtomicI32::new(-1); // -1 is off
static protocol_version: AtomicI32 = AtomicI32::new(777); // 777 is 26.3

// craft packets

struct Packet {
    id: u32, // ID/packet type or whatever
    data: Vec<u8> // u8 used for varints
}

// use u32 becaus apparently claude said integer_encoding uses zigzag encodign on i32, and already
// our server only uses positive values or whatever yeah meheh

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
        let threshold = compression_threshold.load(Ordering::Relaxed);
        // create a vec w/ the id
        let mut packet_data: Vec<u8> = Vec::new();
        // but data length first!! lazy asf
        if threshold > -1 {
            packet_data.write_varint(0u32).unwrap();
        }
        packet_data.write_varint(self.id as u32).unwrap();
        packet_data.extend_from_slice(&self.data);

        // now create the full packet
        let mut full_packet: Vec<u8> = Vec::new();
        full_packet.write_varint(packet_data.len() as u32).unwrap();
        // uncompresed packets
        //if threshold > -1 {
        //    full_packet.write_varint(0u32).unwrap();
        //}
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

// packet handling

async fn read_packet(stream: &mut TcpStream) -> std::io::Result<(u32, Vec<u8>)> {
    let threshold = compression_threshold.load(Ordering::Relaxed); // get compression threshold
    let packet_length: u32 = stream.read_varint_async().await?; // Get packet length
    let mut data_length: u32 = 0;
    //if threshold > -1 { // if compression is enabled
    //    data_length = stream.read_varint_async().await?; // get data length
    //}
    let mut server_packet_data = vec![0u8; packet_length as usize]; // Create vec w/ packet
                                                                             // length as size
    stream.read_exact(&mut server_packet_data).await?; // and read the exact length of bytes provided by
                                                   // server!!
    // here we parse the packet
    
    let mut cur = &server_packet_data[..]; // take the um data
    // data length in compression setups
    if threshold > -1 {
        data_length = cur.read_varint()?;
    }
    let mut decompressed_packet_data = Vec::with_capacity(data_length as usize);
    if data_length != 0 {
        //panic!("Compressed data! rustyclient does not support compressed packets yet.");
        //println!("compressed data: {:?}", server_packet_data);
        let mut z = ZlibDecoder::new(&cur[..]);
        z.read_to_end(&mut decompressed_packet_data)?;
        let mut cur = &decompressed_packet_data[..];
        println!("decompressed data len: {0} expected len: {1}", decompressed_packet_data.len(), data_length);
        let id: u32 = cur.read_varint()?;
        // println!("Compressed packet id (Likely config/postconfig state): {id}");
        Ok((id, cur.to_vec()))
    } else {
        let id: u32 = cur.read_varint()?; // read packet id
        // println!("Decompressed packet id (Likely login state): {id}");
        Ok((id, cur.to_vec()))
    }
}

// easier packet sending!!
// this handles a handshake

// later
// async fn handshake(stream: &mut stream)

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
    pk.data.write_varint(2u32).unwrap(); // 2 for Login
    pk.send(&mut stream).await?;
    println!("Handshake sent!");
    println!("Login Start packet");
    //
    // Status Request
    // No fields, just craft a basic packet w/ 0x00 and youre done. You can send ping + pong but not
    // needed for a beginner project
    //let mut pk = Packet::new(0x00);
    //pk.send(&mut stream).await?;
    //let (id, fbody) = read_packet(&mut stream).await?;
    //let mut cur = &fbody[..];
    //let json_len: u32 = cur.read_varint()?;
    //let json = String::from_utf8_lossy(&cur[..json_len as usize]);
    //println!("{id}: {json}");

    // Login Start (0x00)
    // String (write_string): Player's username (16char max)
    // UUID: just put like a 16 byte uuid shits in here idfk
    let mut login = Packet::new(0x00);
    // login.data.write_varint(777u32).unwrap(); // should prob make this auto-write ig NO I SHOULD
    // NOT
    login.write_string("rustyclient");
    // dumbass gemini
    //pk.data.extend(repeat(0x01).take(16));
    login.data.extend_from_slice(&[0u8; 16]);
    login.send(&mut stream).await?;
    println!("login start id {1}: {:?}", login.data, login.id);
    println!("raw packet of login start: {:?}", login.to_bytes());

    // recieve Login Success packet
    let (mut id, mut pck) = read_packet(&mut stream).await?;
    match id {
        0x02 => {}, // expected packet, no need to handle
        0x03 => { // set compression packet
            println!("raw data of Set Compression packet: {:?}", &pck);
            let mut cur = &pck[..];
            let threshold = cur.read_varint::<u32>()? as i32;
            compression_threshold.store(threshold, Ordering::Relaxed);
            (id, pck) = read_packet(&mut stream).await?; // now thelogin success packet
        },
        _ => {} // idfk
    }
    let t = compression_threshold.load(Ordering::Relaxed);
    println!("compression threshold: {t}");
    if id == 0x02 {
        println!("login success packet successful");
    } else if id == 0x00 {
        let mut cur = &pck[..];
        let json_len: u32 = cur.read_varint()?;
        let json = String::from_utf8_lossy(&cur[..json_len as usize]);
        println!("login disconnect! {id} {json}");
        println!("raw bytes for debug: {:?}", pck);
        panic!("Unexpected disconnect");
    } else {
        println!("unexpected packet: {id}")
    }
    
    // Login Acknowledged
    // 0x03 w/o fields
    let mut login_ack = Packet::new(0x03);
    login_ack.send(&mut stream).await?;

    let mut login = false;

    while login != true {
        let (config_id, config_pck) = read_packet(&mut stream).await?;
        if config_id == 0x00 { // disconnect
            // println!("configuration disconnect! {1} {:?}", config_pck, config_id);
            let mut cur = &config_pck[..];
            let json_len: u32 = cur.read_varint()?;
            let json = String::from_utf8_lossy(&cur[..json_len as usize]);
            println!("Disconnect reason: {json}");
            panic!("Unexpected disconnect");
        } else if config_id == 0x0F {
            println!("server known packs packet, replying");
            let mut known_packs = Packet::new(0x07);
            known_packs.data.extend_from_slice(&config_pck[..]);
            // known_packs.data.push(0); // empty array
            known_packs.send(&mut stream).await?;
        } else if config_id == 0x03 {
            println!("Configuration done!");
            let config_ack = Packet::new(0x03);
            config_ack.send(&mut stream).await?;
            login = true;
        }
    }

    let (id, pck) = read_packet(&mut stream).await?;
    if id == 0x32 {
        println!("Successfully recieved Login (play) packet!")
    } else {
        panic!("Unexpected packet")
    }
    // test
    let (id, pck) = read_packet(&mut stream).await?;
    if id == 0xA {
        match pck[0] {
            0 => {println!("Peaceful difficulty")},
            1 => {println!("Easy difficulty")},
            2 => {println!("Normal difficulty")},
            3 => {println!("Hard difficulty")}
            _ => {println!("idfk diffuclty")}
        }
    } else {
        println!("ohg nvm");
    }

    stream.shutdown().await?;
    Ok(())
}
