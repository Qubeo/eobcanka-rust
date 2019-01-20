extern crate pcsc;
extern crate bytes;

use pcsc::*;
use bytes::{Bytes, BytesMut, Buf, BufMut, BigEndian};
use std::fmt;
use std::str;

pub mod mycard;

pub struct Application {
    app_id: Vec<u8>
}

impl Application {
    pub const CMD_SELECT_APP_PREFIX: [u8; 3] = [0x00, 0xA4, 0x0C];

    pub fn construct_select_app_cmd() -> Vec<u8> {
        vec![0x00]
    }
}

pub fn select_applet(app_id: Vec<u8>) -> Vec<u8> {
    vec![0x00]
}

pub fn get_card_number(card: pcsc::Card) -> Vec<u8> {

    let result_data: Vec<u8> = vec![0x00];
    result_data
}

pub fn get_serial_number() -> Vec<u8> {
    vec![0xAA]
}

pub fn get_SW(buffer: &Vec<u8>) -> Vec<u8> {
    
    if buffer.len() > 2 {
        vec![get_SW1(&buffer), get_SW2(&buffer)]
    }
    else {
        vec![0x00, 0x00]
    }
}

pub fn get_SW1(buffer: &Vec<u8>) -> u8 {
    if buffer.len() > 1 {
        buffer[buffer.len() - 2]
    }
    else { 0 }
}

pub fn get_SW2(buffer: &Vec<u8>) -> u8 {
    if buffer.len() > 0 {
        buffer[buffer.len() - 1]
    }
    else { 0 }
}

pub fn get_data(card: &pcsc::Card, tag_id: u8, auth_id: u8) -> Vec<u8> {
    
    let auth_id = auth_id << 4;
    let mut request = vec![
                0x80,
                0xCA,
                (auth_id | 1),
                (auth_id | tag_id),
                0
    ];

    let mut response_buf = [0; MAX_BUFFER_SIZE];
    // let mut response_buf = BytesMut::with_capacity(256);

    let mut response = card.transmit(&request, &mut response_buf);
    println!("Response of first transmit: {:x?}", response.unwrap());

    let mut response_vec = response.unwrap().to_vec();
    
    if get_SW(&response_vec) == [0x90, 0x00] {
        println!("Got 0x90, 0x00 right in get_SW!");
        return response_vec;
    }
    else if get_SW1(&response_vec) == 0x6C {
        println!("Got 0x6C in SW1!");
        let request_len = request.len();
        request[request_len - 1] = get_SW2(&response_vec);       // Q: Isn't this a nonsense? Shouldn't reverse the array?

        response = card.transmit(&request, &mut response_buf);
        response_vec = response.unwrap().to_vec();

        if get_SW(&response_vec) == [0x90, 0x00] {
            println!("Got 0x90, 0x00 after SW1 was 0x6C!");
            return response_vec;
        }
    }    

    // println!("get_SW2(): {:x?}", get_SW2(&response_buf.to_vec()));
    // if get_SW

    /*        IResponseAPDU r = c.transmit(c.createCommand(request));
        if (r.getSW() == 0x9000) {
            return r.getData();
        } else if (r.getSW1() == 0x6c) {
            request[request.length - 1] = (byte) r.getSW2();
            r = c.transmit(c.createCommand(request));
            if (r.getSW() == 0x9000) {
                return r.getData();
            }
        }
        return null; */
    
    return response_vec;
} 

pub fn select_application() {

}


fn main() {

    // TODO: Put to a method of MyCard
    let app = Application { app_id: mycard::MyCard::APP_ID_CARD_MANAGEMENT.to_vec() };
    let current_application: Vec<u8> = mycard::MyCard::APP_ID_CARD_MANAGEMENT.to_vec();

    let ctx = match Context::establish(Scope::User) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Failed to establish context: {}", err);
            std::process::exit(1);
        }
    };

    // List available readers.
    let mut readers_buf = [0; 2048];
    let mut readers = match ctx.list_readers(&mut readers_buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("Failed to list readers: {}", err);
            std::process::exit(1);
        }
    };

    // Use the first reader.
    let reader = match readers.next() {
        Some(reader) => reader,
        None => {
            println!("No readers are connected.");
            return;
        }
    };
    println!("Using reader: {:?}", reader);

    // Connect to the card.
    let card = match ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
        Ok(card) => card,
        Err(Error::NoSmartcard) => {
            println!("A smartcard is not present in the reader.");
            return;
        }
        Err(err) => {
            eprintln!("Failed to connect to card: {}", err);
            std::process::exit(1);
        }
    };

    // Specific card interactions -----------------------------------------------------------------
    
    //                           |_C_| |_I_| |P1_| |P2_| |_L_| |________________APP_ID_MANAGEMENT__________________|
    let cmd_select_applet_mgmt = [0x00, 0xA4, 0x04, 0x0C, 0x09, 0xD2, 0x03, 0x10, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02];
    let cmd_select_applet_file = [0x00, 0xA4, 0x04, 0x0C, 0x09, 0xD2, 0x03, 0x10, 0x01, 0x00, 0x01, 0x03, 0x02, 0x01, 0x00];

    // select_application(&card, mycard::MyCard::APP_ID_CARD_MANAGEMENT);

    // Select MGMT applet:
    let mut result_apdu_buf = [0; MAX_BUFFER_SIZE];
    // let result_apdu = card.transmit(&cmd_select_applet_mgmt, &mut result_apdu_buf);
    let result_apdu = card.transmit(&cmd_select_applet_mgmt, &mut result_apdu_buf);
    
    // Get card number
    // TODO: To a function -> to a MyCard method.
    let card_number_request = get_data(&card, mycard::MyCard::TAG_ID_CARD_NUMBER, 0);

    //let card_number_str = str::from_utf8(&card_number_request);
    println!("Card number: {:x?}", card_number_request); // result_card_number_buf.to_vec());

    // Get certificate
    // TODO: File ID
    // let result_certificate = get_file(&card, mycard::MyCard::FILE_ID_CERTIFICATE_IDENTIFICATION);
    // println!("Result certificate: {:?}: ", result_certificate);
    
    /*
    println!("Sending APDU: {:x?}", apdu);
    let mut result_apdu_buf = [0; MAX_BUFFER_SIZE];

    let result_apdu = match card.transmit(&apdu, &mut result_apdu_buf) {
        Ok(result_apdu) => result_apdu,
        Err(err) => {
            eprintln!("Failed to transmit APDU command to card: {}", err);
            std::process::exit(1);
        }
    };
    println!("APDU response: {:x?}", result_apdu);
    */ 
}

/// @get_file
/// @argument file_id: u32
///     file_id is ????
/// @argument card
///     pcsc::Card struct
pub fn get_file(card: &pcsc::Card, file_id: u32) -> Vec<u8> {

    let mut h = file_id / 256;
    let mut l = file_id % 256;
    let cmd_file_info = [0x00, 0xA4, 0x08, 0x00, 0x02, h as u8, l as u8];

    let mut response_buffer = [0; MAX_BUFFER_SIZE];
    let mut response_APDU = card.transmit(&cmd_file_info, &mut response_buffer);
    let mut response_vec = response_APDU.unwrap().to_vec();

    if get_SW1(&response_vec) == 0x61 {
        let cmd_get_response = [0x00, 0xC0, 0x00, 0x00, get_SW2(&response_vec) as u8];
        let response_APDU = card.transmit(&cmd_get_response, &mut response_buffer);
        let response_vec = response_APDU.unwrap().to_vec();
    }

    if get_SW(&response_vec) == [0x90, 0x00] {
        let mut size = 0xD0;
        let mut offset: u32 = 0;

        let file_info_data = response_vec.clone();
        let file_size: u32 = file_info_data[4] as u32 * 256 + file_info_data[5] as u32;

        let mut bos: Vec<u8> = vec![0x00];

        loop {
                println!("Looping...");
                h = offset / 256;
                l = offset % 256;

                if (offset + size) > file_size {
                    size = file_size - offset;
                }
                if (size <= 0) {
                    break;
                }

                let cmd_read_file_request = [0x00, 0xB0, h as u8, l as u8, size as u8];
                response_APDU = card.transmit(&cmd_read_file_request, &mut response_buffer);
                response_vec = response_APDU.unwrap().to_vec();

                if get_SW(&response_vec) == [0x90, 0x00] {
                    let mut to_append = response_vec.clone();
                    bos.append(&mut to_append);
                    offset += size;
                } 

                if get_SW(&response_vec) != [0x90, 0x00] { break; }
        }

        return bos;
    }
    return vec![0x00];
}