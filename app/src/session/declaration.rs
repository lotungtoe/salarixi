use std::sync::Arc;

use once_cell::sync::Lazy;
use salarixi_kernel::server::Server;
use salarixi_kernel::sleep;
use salarixi_kernel::tools::{randnum, randstr, CharClass};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::emit::EMIT_MANAGER;
use crate::result::CommandResult;
use crate::session::client_process;
use crate::session::transform::transform_id;
use crate::{failed, success};

pub static SESSION: Lazy<Session> = Lazy::new(|| Session::new());

pub struct Session {
  local_server: RwLock<Option<Server>>,
  reader_task: RwLock<Option<JoinHandle<()>>>,
  reader: Arc<RwLock<Option<OwnedReadHalf>>>,
  writer: Arc<RwLock<Option<OwnedWriteHalf>>>,
}

impl Session {
  pub fn new() -> Self {
    Self {
      local_server: RwLock::new(None),
      reader_task: RwLock::new(None),
      reader: Arc::new(RwLock::new(None)),
      writer: Arc::new(RwLock::new(None)),
    }
  }

  /// Метод запуска локального сервера
  pub async fn run_local_server(&self) -> CommandResult<(String, String)> {
    self.shutdown().await;

    let mut data = None;
    let mut error = None;
    let random_password = randstr(CharClass::Multi, randnum(16, 24));

    for port in 30000..=50000 {
      let addr = format!("127.0.0.1:{}", port);
      match Server::run(&addr, Some(random_password.clone())).await {
        Ok(s) => {
          data = Some((s, addr));
          break;
        }
        Err(e) => {
          error = Some(e);
        }
      }
    }

    if let Some((server, addr)) = data {
      server.start_processing().await;
      *self.local_server.write().await = Some(server);
      sleep!(1000);

      let result = self
        .connect_to_remote_server(addr.clone(), Some(random_password.clone()), false)
        .await;

      if let Some(e) = result.error {
        failed!("{}", e);
      } else {
        success!((addr, random_password));
      }
    } else {
      if let Some(e) = error {
        failed!("{}", e);
      } else {
        failed!("unknown error occurred while starting server");
      }
    }
  }

  /// Метод подключения к удалённому серверу
  pub async fn connect_to_remote_server(
    &self,
    addr: String,
    password: Option<String>,
    is_changing: bool,
  ) -> CommandResult<()> {
    if is_changing {
      self.shutdown().await;
    }

    let mut socket = match TcpStream::connect(addr).await {
      Ok(s) => s,
      Err(_) => {
        failed!("target server is not responding");
      }
    };

    if let Err(e) = client_process(&mut socket, password).await {
      failed!("{}", e);
    }

    let (r, w) = socket.into_split();

    *self.reader.write().await = Some(r);
    *self.writer.write().await = Some(w);

    self.run_reader_task().await;

    success!(());
  }

  /// Метод выключения сессии
  pub async fn shutdown(&self) {
    let mut task_guard = self.reader_task.write().await;
    if let Some(t) = task_guard.as_ref() {
      t.abort();
    }

    *task_guard = None;
    drop(task_guard);

    let mut writer_guard = self.writer.write().await;
    if let Some(w) = writer_guard.as_mut() {
      let _ = w.shutdown().await;
    }

    *writer_guard = None;
    drop(writer_guard);

    *self.reader.write().await = None;

    let mut server_guard = self.local_server.write().await;
    if let Some(s) = server_guard.as_ref() {
      s.stop_processing().await;
      // TODO: Грамотная и тихая остановка ботов (если процесс активен)
    }

    *server_guard = None;
    drop(server_guard);
  }

  /// Метод запуска цикла чтения входящих пакетов
  async fn run_reader_task(&self) {
    let reader = self.reader.clone();
    let task = tokio::spawn(async move {
      let read_fn = async |half: &mut OwnedReadHalf| {
        let id = half.read_u8().await.ok()?;
        let len = half.read_u32().await.ok()?;
        let mut data = Vec::with_capacity(len as usize);

        for _ in 0..len {
          data.push(half.read_u8().await.ok()?);
        }

        Some((id, data))
      };

      loop {
        let mut reader_guard = reader.write().await;
        let Some(half) = reader_guard.as_mut() else {
          continue;
        };

        let Some((id, data)) = read_fn(half).await else {
          continue;
        };

        let transformed_id = transform_id(id);

        EMIT_MANAGER.emit(transformed_id, data);
      }
    });

    *self.reader_task.write().await = Some(task);
  }

  /// Метод отправки команды
  pub async fn send_command(&self, data: Vec<u8>) -> CommandResult<()> {
    if let Some(writer) = self.writer.write().await.as_mut() {
      match writer.write_all(&data).await {
        Ok(_) => success!(()),
        Err(e) => failed!("write error: {}", e),
      }
    }

    failed!("session not established");
  }
}
