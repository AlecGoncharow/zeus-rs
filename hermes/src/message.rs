use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Pulled type requires {type_size}, but the Message body only has {remaining} bytes.")]
    NotEnoughBytes { type_size: usize, remaining: usize },
}

pub trait Pod: 'static + Copy + Sized + Send + Sync + std::fmt::Debug {}

impl<T: 'static + Copy + Sized + Send + Sync + std::fmt::Debug> Pod for T {}

pub trait Messageable: Pod {}

/// T represents should probably be a Enum which tells both sides what kind of message is being
/// passed in the body of the message
#[derive(Debug, Clone, Copy)]
pub struct MessageHeader<T: Messageable> {
    /// Type of message`
    pub id: T,
    /// the length of the message in bytes
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Message<T: Messageable> {
    pub header: MessageHeader<T>,
    pub body: Vec<u8>,
}

impl<T: Messageable> Message<T> {
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
            // SAFETY:
            // Since we resized the Vec to fit the existing data along with the number of bytes in
            // the incoming type, we can be sure that there is enough space to copy the actual
            // data
            // The source is known to be aligned since it is just a reinterpetation of an existing
            // valid Pod type
            std::ptr::copy(
                byte_ptr,
                self.body.as_mut_ptr().add(i),
                std::mem::size_of::<V>(),
            );
        }

        self.header.size = self.size();
    }

    /// This will reinterpet the end of the message body's bytes as the type you request,
    /// there is no built in validation at the moment.
    pub fn pull<V: Pod>(&mut self) -> Result<V, MessageError> {
        let bytes = std::mem::size_of::<V>();
        if bytes > self.body.len() {
            return Err(MessageError::NotEnoughBytes {
                type_size: bytes,
                remaining: self.body.len(),
            });
        }

        let new_len = self.body.len() - bytes;

        let out: V = unsafe {
            // SAFETY:
            // We know that the slice of bytes has the proper number of bytes to transmute into an
            // instance of `V` due to the above code deriving the values based on the `size_of` calls
            // on `V`
            //
            // Caveat:
            // This call will reinterpet the bytes as an instance of `V`, there is currently not
            // parity check on the result of this reinterpetation, the burden of doing some
            // validation is currently on the caller
            self.body
                .as_ptr()
                .offset(new_len as isize)
                .cast::<V>()
                .read_unaligned()
        };

        self.body.resize(new_len, 0);

        self.header.size = self.size();

        Ok(out)
    }
}

impl<T: Messageable> std::fmt::Display for Message<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID:{:?} Size:{:?}", self.header.id, self.header.size)
    }
}

impl<T: Messageable> From<Message<T>> for Vec<u8> {
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

impl<T: Messageable> From<&[u8]> for Message<T> {
    fn from(bytes: &[u8]) -> Self {
        let header_size = std::mem::size_of::<MessageHeader<T>>();
        let bytes_len = bytes.len();
        if bytes_len < header_size {
            panic!("no, this is not header");
        }

        let header: MessageHeader<T> =
            unsafe { bytes.as_ptr().cast::<MessageHeader<T>>().read_unaligned() };

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

impl<T: Messageable> From<&[u8]> for MessageHeader<T> {
    fn from(bytes: &[u8]) -> Self {
        let header_size = std::mem::size_of::<MessageHeader<T>>();
        let bytes_len = bytes.len();
        if bytes_len < header_size {
            panic!("no, this is not header");
        }

        unsafe { std::mem::transmute_copy(&bytes[0]) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
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

    impl Messageable for CustomMsg {}

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
    fn messages() -> Result<()> {
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

        let out_d: [F2; 2] = message.pull()?;
        let out_c: f32 = message.pull()?;
        println!("{:?}", message);
        let out_b: bool = message.pull()?;
        println!("{:?}", message);
        let out_a: u32 = message.pull()?;
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
        let out_complex = message.pull::<Complex>()?;
        println!("{:?}", message);
        println!("out_complex: {:#?}", out_complex);
        assert_eq!(complex.a, out_complex.a);
        assert_eq!(complex.b, out_complex.b);
        assert_eq!(complex.c, out_complex.c);
        Ok(())
    }
}
