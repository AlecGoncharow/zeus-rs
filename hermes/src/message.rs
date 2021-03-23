pub trait Pod: 'static + Copy + Sized + Send + Sync + std::fmt::Debug {}

impl<T: 'static + Copy + Sized + Send + Sync + std::fmt::Debug> Pod for T {}

pub trait MessageKind: Pod {}

/// T represents an Enum which tells both sides what kind of message is being
/// passed in the body of the message
#[derive(Debug, Clone, Copy)]
pub struct MessageHeader<T: MessageKind> {
    /// The kind of invariant in the message body, used as an identifier
    pub id: T,
    /// the length of the message in bytes
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Message<T: MessageKind> {
    pub header: MessageHeader<T>,
    pub body: Vec<u8>,
}

impl<T: MessageKind> Message<T> {
    pub fn new(id: T) -> Self {
        let header = MessageHeader {
            id,
            size: std::mem::size_of::<MessageHeader<T>>() as u32,
        };

        Self {
            header,
            body: vec![],
        }
    }

    pub fn size(&self) -> u32 {
        (std::mem::size_of::<MessageHeader<T>>() + self.body.len()) as u32
    }

    pub fn push<V: Pod>(&mut self, data: V) {
        let i = self.body.len();

        self.body
            .resize(self.body.len() + std::mem::size_of::<V>(), 0);

        unsafe {
            let data_ptr: *const V = &data;
            let byte_ptr: *const u8 = data_ptr as *const _;
            let byte_slice: &[u8] = std::slice::from_raw_parts(byte_ptr, std::mem::size_of::<V>());

            std::ptr::copy(
                &byte_slice[0],
                self.body.as_mut_ptr().offset(i as isize),
                std::mem::size_of::<V>(),
            );
        }

        self.header.size = self.size();
    }

    pub fn pull<V: Pod>(&mut self) -> V {
        let bytes = std::mem::size_of::<V>();
        let i = self.body.len() - bytes;

        let out = unsafe {
            let data_ptr = self.body.as_ptr().offset(i as isize);
            let byte_slice: &[u8] = std::slice::from_raw_parts(data_ptr, bytes);

            std::mem::transmute_copy(&byte_slice[0])
        };

        self.body.resize(i, 0);

        self.header.size = self.size();

        out
    }
}

impl<T: MessageKind> std::fmt::Display for Message<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID:{:?} Size:{:?}", self.header.id, self.header.size)
    }
}

impl<T: MessageKind> From<Message<T>> for Vec<u8> {
    fn from(msg: Message<T>) -> Self {
        let header_size = std::mem::size_of::<MessageHeader<T>>();
        let body_len = msg.body.len();
        let mut out = Vec::with_capacity(header_size + body_len);

        unsafe {
            let data_ptr: *const MessageHeader<T> = &msg.header;
            let byte_ptr: *const u8 = data_ptr as *const _;
            let byte_slice: &[u8] = std::slice::from_raw_parts(byte_ptr, header_size);

            out.extend_from_slice(&byte_slice);

            if body_len > 0 {
                out.extend_from_slice(&msg.body);
            }
            out
        }
    }
}

impl<T: MessageKind> From<&[u8]> for Message<T> {
    fn from(bytes: &[u8]) -> Self {
        let header_size = std::mem::size_of::<MessageHeader<T>>();
        let bytes_len = bytes.len();
        if bytes_len < header_size {
            panic!("no, this is not header");
        }

        let header: MessageHeader<T> = unsafe { std::mem::transmute_copy(&bytes[0]) };

        if header.size != bytes_len as u32 {
            panic!(
                "Expected size:{} in header, got:{}",
                bytes.len(),
                header.size
            );
        }

        let body_bytes = &bytes[header_size..];
        let body = Vec::from(body_bytes);

        Self { header, body }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[derive(Clone, Copy, Debug)]
    pub enum CustomMsg {
        Interact(usize),
        #[allow(dead_code)]
        MovePlayer(usize),
    }

    impl std::fmt::Display for CustomMsg {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CustomMsg::Interact(id) => write!(f, "Interact({})", id),
                CustomMsg::MovePlayer(id) => write!(f, "MovePlayer({})", id),
            }
        }
    }

    impl MessageKind for CustomMsg {}

    #[derive(Clone, Copy, Debug)]
    struct F2 {
        x: f32,
        y: f32,
    }

    #[derive(Clone, Copy, Debug)]
    struct Complex {
        a: u32,
        b: bool,
        c: f32,
        d: [F2; 2],
    }

    #[test]
    fn messages() {
        let id = CustomMsg::Interact(23);
        let mut message = Message::new(id);

        let a = 2;
        let b = true;
        let c: f32 = 3.14159;
        println!("a; {:?}", a);
        println!("b: {:?}", b);
        println!("c: {:?}", c);

        let d = [
            F2 { x: 1., y: 2. },
            F2 {
                x: 1.243,
                y: -1234.1,
            },
        ];

        println!("d: {:?}", d);

        println!("{:?}", message);
        message.push(a);
        println!("{:?}", message);
        message.push(b);
        println!("{:?}", message);
        message.push(c);
        println!("{:?}", message);
        message.push(d);
        println!("{:?}", message);

        let out_d: [F2; 2] = message.pull();
        let out_c: f32 = message.pull();
        println!("{:?}", message);
        let out_b: bool = message.pull();
        println!("{:?}", message);
        let out_a: u32 = message.pull();
        //let (out_d, out_c, out_b, out_a): ([F2; 2], f32, bool, u32) = message.pull(25);
        //let (out_a, out_b, out_c, out_d): (u32, bool, f32, [F2; 2]) = message.pull(25);
        println!("{:?}", message);
        println!("out_a; {:?}", out_a);
        println!("out_b: {:?}", out_b);
        println!("out_c: {:?}", out_c);
        println!("out_d: {:?}", out_d);

        assert_eq!(a, out_a);
        assert_eq!(b, out_b);
        assert_eq!(c, out_c);

        let complex = Complex { a, b, c, d };
        println!("complex: {:#?}", complex);
        message.push(complex);
        println!("{:?}", message);
        let out_complex = message.pull::<Complex>();
        println!("{:?}", message);
        println!("out_complex: {:#?}", out_complex);
        assert_eq!(complex.a, out_complex.a);
        assert_eq!(complex.b, out_complex.b);
        assert_eq!(complex.c, out_complex.c);
    }
}
