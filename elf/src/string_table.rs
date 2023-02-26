// This module abstracts ELF string tables, making them easier
// to use and build. StrTab does not do fancy string interning
// so 'carpet' and 'pet' will not overlap on disk unlike many
// implementations, but it gets the job done for simple tables.

#[derive(Debug)]
pub struct StrTab {
    bytes: Vec<u8>
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

    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Default for StrTab {
    fn default() -> Self {
        StrTab::new(&[0 as u8])
    }
}


#[cfg(test)]
mod tests {
    use crate::string_table::StrTab;

    #[test]
    fn default() {
        let mut st = StrTab::default();

        assert_eq!(st.len(), 1);

        let foo = st.insert("foo");
        let bar = st.insert("bar");

        assert_eq!(st.len(), 9);

        assert_eq!(st.at(foo), Some("foo".to_string()));
        assert_eq!(st.at(bar), Some("bar".to_string()));
    }

    #[test]
    fn from_buf() {
        let buf : Vec<u8> = vec![0,65, 66, 67, 0];
        let st = StrTab::new(&buf);
        assert_eq!(st.at(0), Some("".to_string()));
        assert_eq!(st.at(1), Some("ABC".to_string()));
    }
}