use std::io::{Error, ErrorKind};
use std::sync::Arc;

use bytes::BytesMut;
use hashbrown::HashMap;
use salarixi_extensions::index::IndexExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::commands::{execute_client_command, ClientCommand};
use crate::server::process::process_socket;
use crate::server::transfer::{TransferEvent, TRANSFER};
use crate::server::write_bytes_to_sockets;

pub struct Server {
  pub listener: Arc<RwLock<TcpListener>>,
  password: Arc<Option<String>>,
  sockets: Arc<RwLock<HashMap<String, (Arc<RwLock<OwnedReadHalf>>, Arc<RwLock<OwnedWriteHalf>>)>>>,
  tasks: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl Server {
  /// Метод запуска сервера с указанным паролем
  pub async fn run(addr: impl Into<String>, password: Option<impl Into<String>>) -> std::io::Result<Self> {
    let mut pass = None;

    if let Some(pass_impl) = password {
      let p = pass_impl.into();

      if p.len() > 255 {
        return Err(Error::new(
          ErrorKind::InvalidData,
          "password length cannot be more than 255",
        ));
      }

      if p.len() < 8 {
        println!("[warning] password less than 8 chars may be weak");
      }

      pass = Some(p);
    }

    let listener = TcpListener::bind(addr.into()).await?;

    Ok(Self {
      listener: Arc::new(RwLock::new(listener)),
      password: Arc::new(pass),
      sockets: Arc::new(RwLock::new(HashMap::new())),
      tasks: Arc::new(RwLock::new(Vec::new())),
    })
  }

  /// Метод запуска обработки входящих соединений
  pub async fn start_processing(&self) {
    let listener = self.listener.clone();
    let password = self.password.clone();
    let sockets = self.sockets.clone();
    let tasks = self.tasks.clone();
    let root_task = tokio::spawn(async move {
      loop {
        match listener.read().await.accept().await {
          Ok((mut socket, addr)) => {
            let addr_string = addr.to_string();

            println!("[info] new connection from {}", addr_string);

            match process_socket(&mut socket, &addr_string, &password).await {
              Ok(_) => {
                println!("[success :: {}] connection registered successfully", addr_string);

                let (r, w) = socket.into_split();
                let reader = Arc::new(RwLock::new(r));
                let writer = Arc::new(RwLock::new(w));

                sockets
                  .write()
                  .await
                  .insert(addr_string.clone(), (reader.clone(), writer.clone()));

                let read_loop = tokio::spawn(Self::run_individual_read_loop(
                  sockets.clone(),
                  addr_string,
                  reader,
                  writer,
                ));
                tasks.write().await.push(read_loop);
              }
              Err(e) => {
                println!("[error :: {}] connection broken: {}", addr_string, e);
                let _ = socket.shutdown().await;
              }
            }
          }
          Err(e) => {
            println!("[error] could not accept incoming connection: {}", e);
          }
        }
      }
    });

    let mut emit_rx = TRANSFER.tx.subscribe();
    let sockets = self.sockets.clone();
    let write_task = tokio::spawn(async move {
      loop {
        if let Ok(event) = emit_rx.recv().await {
          let bytes;

          match event {
            TransferEvent::Log(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::Message(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::ConnectedBots(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::SessionChat(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::TotalBots(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::BotChat(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::AntiWebCaptcha(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::AntiMapCaptcha(payload) => {
              bytes = payload.to_bytes();
            }
            TransferEvent::ProcessStatus(payload) => {
              bytes = payload.to_bytes();
            }
          }

          write_bytes_to_sockets(&sockets, bytes).await;
        }
      }
    });

    let mut tasks_guard = self.tasks.write().await;
    tasks_guard.push(root_task);
    tasks_guard.push(write_task);
    drop(tasks_guard);

    println!("[info] server started processing incoming connections");
  }

  /// Метод остановки обработки входящих соединений
  pub async fn stop_processing(&self) {
    let mut tasks_guard = self.tasks.write().await;
    tasks_guard.iter().for_each(|t| t.abort());
    tasks_guard.clear();
    drop(tasks_guard);

    self.sockets.write().await.clear();

    println!("[info] server stopped processing incoming connections");
  }

  /// Метод запуска индивидуального цикла чтения пакетов указанного сокета
  async fn run_individual_read_loop(
    sockets: Arc<RwLock<HashMap<String, (Arc<RwLock<OwnedReadHalf>>, Arc<RwLock<OwnedWriteHalf>>)>>>,
    addr: String,
    reader: Arc<RwLock<OwnedReadHalf>>,
    writer: Arc<RwLock<OwnedWriteHalf>>,
  ) {
    loop {
      let mut reader_guard = reader.write().await;
      let Some((id, data)) = Self::read_packet(&mut reader_guard).await else {
        println!("[error :: {}] client sent an invalid packet, read loop finished", addr);
        let _ = writer.write().await.shutdown().await;
        sockets.write().await.remove(&addr);
        break;
      };

      drop(reader_guard);

      let Some(command) = ClientCommand::from_index(id) else {
        println!("[error :: {}] unknown command received", addr);
        continue;
      };

      let buf = BytesMut::from_iter(data);

      if let Err(e) = execute_client_command(addr.clone(), &writer, &command, buf, &sockets).await {
        println!("[error :: {}] command \"{:?}\" execution error: {}", addr, command, e);
      }
    }
  }

  /// Вспомогательный метод чтения пакета от клиента
  async fn read_packet(read_half: &mut OwnedReadHalf) -> Option<(u8, Vec<u8>)> {
    let id = read_half.read_u8().await.ok()?;
    let len = read_half.read_u32().await.ok()?;
    let mut data = Vec::with_capacity(len as usize);

    for _ in 0..len {
      data.push(read_half.read_u8().await.ok()?);
    }

    Some((id, data))
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use tokio::net::TcpStream;

  use crate::server::Server;

  #[tokio::test]
  async fn test_server() -> std::io::Result<()> {
    let server = Server::run("127.0.0.1:34457", Some("qwe123")).await?;
    server.start_processing().await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut stream = TcpStream::connect("127.0.0.1:34457").await?;

    stream.write_u8(5).await?;
    stream.write_all("1.1.1".as_bytes()).await?;

    let resp = stream.read_u8().await?;
    println!("handshake response: {}", resp);

    let resp = stream.read_u8().await?;
    println!("password required: {}", resp == 1);

    Ok(())
  }
}
