#[derive(Debug, PartialEq)]
pub enum TokkerPollResult {
    None,
    Data(String),
    Token(String),
}

#[derive(Debug)]
pub struct Tokker {
    in_flight: String,
    data_end: usize,
    token: Vec<String>,
}

impl Tokker {
    pub fn new(token: Vec<String>) -> Self {
        let mut token = token;
        token.sort_by_key(|v| v.len() as isize * -1);

        Self {
            in_flight: String::new(),
            data_end: 0,
            token,
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        self.in_flight.push_str(std::str::from_utf8(data).unwrap());
        self.data_end += data.len();

        let mut keep_end = 0;
        for token in &self.token {
            for l in 1..token.len() {
                if self.in_flight.ends_with(&token[..l]) {
                    keep_end = usize::max(l, keep_end);
                }
            }
        }
        self.data_end = self.in_flight.len() - keep_end;
    }

    pub fn poll(&mut self) -> TokkerPollResult {
        if self.data_end > 0 {
            let res = self.in_flight[..self.data_end].to_string();
            for token in &self.token {
                if res.starts_with(token) {
                    self.in_flight.drain(..token.len());
                    self.data_end -= token.len();
                    return TokkerPollResult::Token(token.to_string());
                }
            }

            let mut found_at_idx = None;
            for token in &self.token {
                if let Some(index) = res.find(token) {
                    if let Some(prev) = found_at_idx {
                        if index < prev {
                            found_at_idx = Some(index);
                        }
                    } else {
                        found_at_idx = Some(index);
                    }
                }
            }

            if let Some(index) = found_at_idx {
                let head = self.in_flight[..index].to_string();
                self.in_flight.drain(..index);
                self.data_end -= head.len();
                return TokkerPollResult::Data(head);
            }

            self.in_flight.drain(..self.data_end);
            self.data_end = 0;
            TokkerPollResult::Data(res)
        } else {
            TokkerPollResult::None
        }
    }
}

#[derive(Debug)]
pub struct Utf8Fixer {
    data: Vec<u8>,
}

impl Utf8Fixer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn poll(&mut self) -> Vec<u8> {
        loop {
            if self.data.len() == 0 {
                break;
            }

            if self.data[0] & 0b1100_0000 == 0b1000_0000 {
                self.data.drain(..1);
            } else {
                break;
            }
        }

        if self.data.len() == 0 {
            return vec![];
        }

        let mut valid = 0;
        loop {
            if self.data[valid] & 0b1000_0000 == 0b0000_0000 {
                // ok - ASCII
                valid += 1;
            } else if self.data[valid] & 0b1100_0000 == 0b1100_0000 {
                // UTF multi byte char first byte
                let skip = self.data[valid].leading_ones() as usize;

                if valid + skip <= self.data.len() {
                    valid += skip as usize;
                } else {
                    break;
                }
            } else if self.data[valid] & 0b1100_0000 == 0b1000_0000 {
                // UTF multi byte char w/o start - will get discarded by the next `poll`
                break;
            }

            if valid >= self.data.len() {
                break;
            }
        }

        let mut res = Vec::new();
        res.extend_from_slice(&self.data[..valid]);
        self.data.drain(..valid);
        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"diwjeodj wefijweoifj weoifjweiojf owefiojwo");
        assert_eq!(
            TokkerPollResult::Data("diwjeodj wefijweoifj weoifjweiojf owefiojwo".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::None, stream.poll());
    }

    #[test]
    fn test1() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"abcd");
        stream.push(b"@CORE");

        assert_eq!(TokkerPollResult::Data("abcd".to_string()), stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());

        stream.push(b"DUMP");
        assert_eq!(TokkerPollResult::None, stream.poll());

        stream.push(b"\n");
        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );

        stream.push(b"foo");
        assert_eq!(TokkerPollResult::Data("foo".to_string()), stream.poll());
    }

    #[test]
    fn test2() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"abcd");
        stream.push(b"@CORE");

        assert_eq!(TokkerPollResult::Data("abcd".to_string()), stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());

        stream.push(b"DUMP");
        stream.push(b"\n");
        stream.push(b"foo");

        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::Data("foo".to_string()), stream.poll());
    }

    #[test]
    fn test3() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"abcd");
        stream.push(b"@CORE");

        assert_eq!(TokkerPollResult::Data("abcd".to_string()), stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());
        assert_eq!(TokkerPollResult::None, stream.poll());

        stream.push(b"abcd");
        assert_eq!(
            TokkerPollResult::Data("@COREabcd".to_string()),
            stream.poll()
        );
    }

    #[test]
    fn test4() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"wehdihweuidhweuihduiwehduiwehduiweh@COREDUMP\nabcdwejdjweiodjoiwejdiowejiowjiodjweiojdiowjediwejwd");
        assert_eq!(
            TokkerPollResult::Data("wehdihweuidhweuihduiwehduiwehduiweh".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Data(
                "abcdwejdjweiodjoiwejdiowejiowjiodjweiojdiowjediwejwd".to_string()
            ),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::None, stream.poll());
    }

    #[test]
    fn test5() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"wehdihweuidhweuihduiwehduiwehduiweh@COREDUMP\nabcdwejdjweiodjoiwejdiowejiowjiodjweiojd@FOOiowjediwejwd");
        assert_eq!(
            TokkerPollResult::Data("wehdihweuidhweuihduiwehduiwehduiweh".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Data("abcdwejdjweiodjoiwejdiowejiowjiodjweiojd".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::Token("@FOO".to_string()), stream.poll());
        assert_eq!(
            TokkerPollResult::Data("iowjediwejwd".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::None, stream.poll());
    }

    #[test]
    fn test6() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@FOO".to_string()]);

        stream.push(b"wehdihweuidhweuihduiwehduiwehduiweh@FOOabcdwejdjweiodjoiwejdiowejiowjiodjweiojd@COREDUMP\niowjediwejwd");
        assert_eq!(
            TokkerPollResult::Data("wehdihweuidhweuihduiwehduiwehduiweh".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::Token("@FOO".to_string()), stream.poll());
        assert_eq!(
            TokkerPollResult::Data("abcdwejdjweiodjoiwejdiowejiowjiodjweiojd".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Data("iowjediwejwd".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::None, stream.poll());
    }

    #[test]
    fn test7() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@ENDCOREDUMP".to_string()]);

        stream.push(b"wehdihweuidhweuihduiwehduiwehduiweh@COREDUMP\nabcdwejdjweiodjoiwejdiowejiowjiodjweiojd");
        assert_eq!(
            TokkerPollResult::Data("wehdihweuidhweuihduiwehduiwehduiweh".to_string()),
            stream.poll()
        );

        stream.push(b"@END");

        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );

        stream.push(b"COREDUMP");

        assert_eq!(
            TokkerPollResult::Data("abcdwejdjweiodjoiwejdiowejiowjiodjweiojd".to_string()),
            stream.poll()
        );
        assert_eq!(
            TokkerPollResult::Token("@ENDCOREDUMP".to_string()),
            stream.poll()
        );
        assert_eq!(TokkerPollResult::None, stream.poll());
    }

    #[test]
    fn test8() {
        let mut stream = Tokker::new(vec!["@COREDUMP\n".to_string(), "@ENDCOREDUMP".to_string()]);

        stream.push(&[b'A'; 1024]);
        stream.push(b"@COREDUMP\n");
        stream.push(&[b'A'; 1024]);

        assert_eq!(
            TokkerPollResult::Data(std::str::from_utf8(&[b'A'; 1024]).unwrap().to_string()),
            stream.poll()
        );

        assert_eq!(
            TokkerPollResult::Token("@COREDUMP\n".to_string()),
            stream.poll()
        );

        assert_eq!(
            TokkerPollResult::Data(std::str::from_utf8(&[b'A'; 1024]).unwrap().to_string()),
            stream.poll()
        );

        stream.push(b"@ENDCOREDUMP\n");

        assert_eq!(
            TokkerPollResult::Token("@ENDCOREDUMP".to_string()),
            stream.poll()
        );

        assert_eq!(TokkerPollResult::Data("\n".to_string()), stream.poll());
    }

    #[test]
    fn test0_utf() {
        let mut v = Utf8Fixer::new();

        v.push(&[240, 159, 146, 150]);
        assert!(v.poll() == &[240, 159, 146, 150]);

        v.push(&[240]);
        assert!(v.poll() == &[]);
        v.push(&[159, 146, 150]);
        assert!(v.poll() == &[240, 159, 146, 150]);

        v.push(&[240]);
        assert!(v.poll() == &[]);
        v.push(&[159]);
        assert!(v.poll() == &[]);
        v.push(&[146]);
        assert!(v.poll() == &[]);
        v.push(&[150]);

        assert!(v.poll() == &[240, 159, 146, 150]);
    }

    #[test]
    fn test1_utf() {
        let mut v = Utf8Fixer::new();

        v.push(&[159, 146, 150]);
        assert!(v.poll() == &[]);

        v.push(b"Hello World");
        assert!(v.poll() == b"Hello World");
        assert!(v.poll() == &[]);
        assert!(v.poll() == &[]);
        assert!(v.poll() == &[]);
    }
}
