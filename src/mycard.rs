extern crate bytes;
extern crate pcsc;

use bytes::*;
use pcsc::*;

pub struct MyCard {
    pub my_card: pcsc::Card,
    pub my_ctx: pcsc::Context,
    //pub mut reader:
}

impl MyCard {
    pub fn new() -> MyCard {
        //let app = Application { app_id: mycard::MyCard::APP_ID_CARD_MANAGEMENT.to_vec() };
        // let current_application: Vec<u8> = mycard::MyCard::APP_ID_CARD_MANAGEMENT.to_vec();

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
        let reader = readers.next().unwrap();
        /* let reader = match readers.next() {
            Some(reader) => reader,
            None => {
                println!("No readers are connected.");
                return _;
            }
        }; */

        println!("Using reader: {:?}", reader);

        // Connect to the card.
        // let my_card = match ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
        let my_card = ctx
            .connect(reader, ShareMode::Shared, Protocols::ANY)
            .unwrap();
        /* {
            Ok(card) => card,
            Err(Error::NoSmartcard) => {
                println!("A smartcard is not present in the reader.");
                return !;
            }
            Err(err) => {
                eprintln!("Failed to connect to card: {}", err);
                std::process::exit(1);
            }
        }; */

        return MyCard {
            my_card: my_card,
            my_ctx: ctx,
        };
    }

    pub const TAG_ID_CARD_NUMBER: u8 = 1;
    pub const TAG_ID_CERTIFICATE_SERIAL_NUMBER: u8 = 2; // Q: Why 2 and not 0x02?
    pub const TAG_ID_KEY_KCV: u8 = 0xC0;
    pub const TAG_ID_KEY_COUNTER: u8 = 0xC1;

    pub const TAG_ID_DOK_STATE: u8 = 0x8B;
    pub const TAG_ID_DOK_TRY_LIMIT: u8 = 0x8C;
    pub const TAG_ID_DOK_MAX_TRY_LIMIT: u8 = 0x8D;

    pub const TAG_ID_IOK_STATE: u8 = 0x82;
    pub const TAG_ID_IOK_TRY_LIMIT: u8 = 0x83;
    pub const TAG_ID_IOK_MAX_TRY_LIMIT: u8 = 0x84;

    // Q: Why are those private in Java?
    pub const APP_ID_CARD_MANAGEMENT: [u8; 9] =
        [0xD2, 0x03, 0x10, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02];
    pub const APP_ID_FILE_MANAGEMENT: [u8; 10] =
        [0xD2, 0x03, 0x10, 0x01, 0x00, 0x01, 0x03, 0x02, 0x01, 0x00];

    pub const FILE_ID_CERTIFICATE_AUTHORIZATION: u32 = 0x0132; // Q: Does the int size matter, beyond satisfying the minimal needed size?
    pub const FILE_ID_CERTIFICATE_IDENTIFICATION: u32 = 0x0001;

    pub fn get_SW(&self, buffer: &Vec<u8>) -> Vec<u8> {

        if buffer.len() > 2 {
            vec![self.get_SW1(&buffer), self.get_SW2(&buffer)]
        } else {
            vec![0x00, 0x00]
        }
    }

    pub fn get_SW1(&self, buffer: &Vec<u8>) -> u8 {
        if buffer.len() > 1 {
            buffer[buffer.len() - 2]
        } else {
            0
        }
    }

    pub fn get_SW2(&self, buffer: &Vec<u8>) -> u8 {
        if buffer.len() > 0 {
            buffer[buffer.len() - 1]
        } else {
            0
        }
    }

    pub fn get_data(&self, tag_id: u8, auth_id: u8) -> Vec<u8> {

        let auth_id = auth_id << 4;
        let mut request = vec![0x80, 0xCA, (auth_id | 1), (auth_id | tag_id), 0];

        let mut response_buf = [0; MAX_BUFFER_SIZE];
        // let mut response_buf = BytesMut::with_capacity(256);

        let mut response = self.my_card.transmit(&request, &mut response_buf);
        println!("Response of first transmit: {:x?}", response.unwrap());

        let mut response_vec = response.unwrap().to_vec();

        if self.get_SW(&response_vec) == [0x90, 0x00] {

            println!("Got 0x90, 0x00 right in get_SW!");
            return response_vec;

        } else if self.get_SW1(&response_vec) == 0x6C {

            println!("Got 0x6C in SW1!");
            let request_len = request.len();
            request[request_len - 1] = self.get_SW2(&response_vec); // Q: Isn't this a nonsense? Shouldn't reverse the array?

            response = self.my_card.transmit(&request, &mut response_buf);
            response_vec = response.unwrap().to_vec();

            if self.get_SW(&response_vec) == [0x90, 0x00] {
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
}

// pub struct ICardInterface { }

impl MyCard {

    // pub fn select_application(app_id: [u8; 8]) -> bool { }

    pub fn select_application(&self, app_id: Vec<u8>) -> bool {

        let cmd_select_applet_mgmt = [0x00, 0xA4, 0x04, 0x0C, 0x09, 0xD2, 0x03, 0x10, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02];        
        let mut result_apdu_buf = [0; MAX_BUFFER_SIZE];
        
        // let result_apdu = card.transmit(&cmd_select_applet_mgmt, &mut result_apdu_buf);
        let result_apdu = self.my_card.transmit(&cmd_select_applet_mgmt, &mut result_apdu_buf);

        return true;
    }

    pub fn get_card_number(&self) -> Vec<u8> {
        
        self.select_application(MyCard::APP_ID_CARD_MANAGEMENT.to_vec());        
        let card_number_request = self.get_data(MyCard::TAG_ID_CARD_NUMBER, 0);
        
        card_number_request.to_vec()
    }

    pub fn create_command(command_data: Vec<u8>) {
        // TODO: How long commandData array? Or bytes? Or Vec<u8>?

    }

    pub fn get_ATR() -> Vec<u8> {
        // TODO: How long array?
        vec![0x00]
    }
}
