#[derive(Default, Debug
)]
pub struct StrTab {
    pub bytes: Vec<u8> // MUST be private
}

impl StrTab {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn at(&self, i: usize) -> Option<String> {
        let mut buffer = String::new();
        for j in i..self.bytes.len() {
            let c = self.bytes[j as usize];
            if c == 0 {
                return Some(buffer.clone());
            }
            buffer.push(c as char);
        }
        None
    }

    pub fn insert(&mut self, s : &str) -> usize {
        let pos = self.bytes.len();
        self.bytes.extend(s.as_bytes());
        self.bytes.push(0);
        pos
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}