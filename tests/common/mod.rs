use lsp_types::Url;
use serde_json::json;

#[derive(Debug)]
struct LspClientInner {
    listener: std::net::TcpListener,
    id: u32,
    stream: Option<std::net::TcpStream>,
    recvbuf: String,
}

impl LspClientInner {
    /// Send request and receive response.
    ///
    /// # Arguments
    ///
    /// + `method` - Method name.
    /// + `params` - Method parameters.
    ///
    /// # Returns
    ///
    /// + Response message.
    pub fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> std::io::Result<serde_json::Value> {
        // Construct the payload.
        let mut msg = self.build_message(method, params);
        msg["id"] = self.id.into();
        self.id += 1;

        // Send request.
        self.send(&msg)?;

        // Rece response.
        let rsp = self.recv().unwrap();

        let obj = rsp.as_object().unwrap();
        if obj.contains_key("result") == false {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                rsp["error"].to_string(),
            ));
        }

        return Ok(rsp["result"].clone());
    }

    /// Send notification.
    ///
    /// # Arguments
    ///
    /// + `method` - Method name.
    /// + `params` - Method parameters.
    pub fn notify(&mut self, method: &str, params: serde_json::Value) -> std::io::Result<()> {
        let msg = self.build_message(method, params);
        self.send(&msg)?;

        Ok(())
    }

    /// Build message.
    ///
    /// # Arguments
    ///
    /// + `method` - Method name.
    /// + `params` - Method parameters.
    ///
    /// # Returns
    ///
    /// + Message.
    fn build_message(&self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let mut msg: serde_json::Value = serde_json::from_str("{\"jsonrpc\": \"2.0\"}").unwrap();
        msg["method"] = method.into();
        msg["params"] = params;

        return msg;
    }

    /// Send message.
    ///
    /// # Arguments
    ///
    /// + `msg` - Message.
    fn send(&mut self, msg: &serde_json::Value) -> std::io::Result<()> {
        use std::io::Write;

        let payload = msg.to_string();
        let data = format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload);

        let mut stream = match &self.stream {
            Some(v) => v,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Connection not initialized",
                ))
            }
        };

        let ret = stream.write(data.as_bytes())?;
        assert!(ret == data.len());

        Ok(())
    }

    /// Receive message.
    ///
    /// # Returns
    ///
    /// + Message.
    fn recv(&mut self) -> std::io::Result<serde_json::Value> {
        use std::io::Read;

        let mut buffer = vec![0; 1024];

        let mut stream = match &self.stream {
            Some(v) => v,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Connection not initialized",
                ))
            }
        };

        // Wait unitil data contains the full header line.
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#contentPart
        loop {
            let read_sz = stream.read(&mut buffer)?;
            let data = String::from_utf8_lossy(&buffer[..read_sz]).to_string();
            self.recvbuf.push_str(data.as_str());

            if self.recvbuf.contains("\r\n\r\n") {
                break;
            }
        }

        // Wait for payload
        let re = regex::Regex::new(r"Content-Length:\s*(\d+)\r\n\r\n").unwrap();
        let cap = re.captures(self.recvbuf.as_str()).unwrap();
        let payload_sz = cap[1].parse::<usize>().unwrap();

        let header_line = cap.get(0).unwrap();
        let header_line_len = header_line.len();
        self.recvbuf = self.recvbuf.chars().skip(header_line_len).collect();

        while self.recvbuf.len() < payload_sz {
            let read_sz = stream.read(&mut buffer)?;
            let data = String::from_utf8_lossy(&buffer[..read_sz]).to_string();
            self.recvbuf.push_str(data.as_str());
        }

        // Parser response
        let data = self.recvbuf.as_str()[..payload_sz].to_string();
        let rsp: serde_json::Value = serde_json::from_str(&data)?;

        // Remove payload from recvbuf.
        self.recvbuf = self.recvbuf.chars().skip(payload_sz).collect();

        return Ok(rsp);
    }
}

#[derive(Debug, Clone)]
pub struct LspClient {
    /// Inner state of the client.
    inner: std::sync::Arc<std::sync::Mutex<LspClientInner>>,
}

impl LspClient {
    /// Create new client.
    ///
    /// # Returns
    ///
    /// + LspClient
    pub fn new() -> std::io::Result<LspClient> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;

        let inner = LspClientInner {
            listener: listener,
            id: 1,
            stream: None,
            recvbuf: "".into(),
        };

        let client = LspClient {
            inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
        };

        return Ok(client);
    }

    /// Get local address.
    ///
    /// # Returns
    ///
    /// + Local address.
    pub fn local_addr(&self) -> std::net::SocketAddr {
        let inner = self.inner.lock().unwrap();
        return inner.listener.local_addr().unwrap();
    }

    /// Perform initialize request.
    pub fn initialize(&mut self) -> std::io::Result<()> {
        let mut inner = self.inner.lock().unwrap();

        let (stream, _) = inner.listener.accept()?;
        inner.stream = Some(stream);

        let root_url = env!("CARGO_TARGET_TMPDIR");
        let root_url = Url::from_file_path(root_url).unwrap();

        let param = json!({
            "rootUri": root_url.to_string(),
            "capabilities": {
                "general": {
                    "positionEncodings": [ "utf-8" ]
                }
            }
        });

        // Send initialize request.
        inner.request("initialize", param)?;

        // Send initialized notification.
        inner.notify("initialized", json!({}))?;

        Ok(())
    }

    /// Perform shutdown request.
    pub fn shutdown(&mut self) -> std::io::Result<()> {
        let mut inner = self.inner.lock().unwrap();

        // Send shutdown request.
        inner.request("shutdown", json!({})).unwrap();

        // Send exit notification.
        inner.notify("exit", json!({}))?;

        Ok(())
    }

    /// Close client.
    pub fn close(&mut self) -> std::io::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.stream = None;
        Ok(())
    }
}
