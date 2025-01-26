use std::io::Read;
use std::marker::{Send, Sync};
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SOCKET_PATH: &str = "/tmp/dwm-statusbar.sock";

type BlockFn = Box<dyn Fn(u32) -> String + Send + Sync>;

pub struct StatusBlock {
    pub f: BlockFn,
    pub interval: Duration,
}

impl StatusBlock {
    pub fn new(f: BlockFn, int_millis: u64) -> Self {
        Self {
            f: Box::new(f),
            interval: Duration::from_millis(int_millis),
        }
    }
}

#[macro_export]
macro_rules! blocks {
    [ $({$f:expr, $int:expr}),* $(,)? ] => {
        vec![$($crate::StatusBlock::new(Box::new($f), $int)),*]
    };
}

pub struct StatusBar {
    separator: &'static str,
    windows: Vec<Vec<Option<StatusBlock>>>,
    // pool: Vec<thread::JoinHandle<()>>,
    placeholder: Arc<Mutex<Vec<String>>>,
    window_index: Arc<Mutex<u32>>,

    tx: mpsc::Sender<()>,
    rx: mpsc::Receiver<()>,
}

impl StatusBar {
    pub fn new(
        separator: &'static str,
        default_window: u32,
        windows: Vec<Vec<StatusBlock>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            placeholder: Arc::new(Mutex::new(vec![
                String::new();
                windows[default_window as usize].len()
            ])),
            separator,
            windows: windows
                .into_iter()
                .map(|i| i.into_iter().map(Some).collect())
                .collect(),
            // pool: vec![],
            window_index: Arc::new(Mutex::new(default_window)),

            tx,
            rx,
        }
    }

    /// Spawn threads, start updates
    pub fn start(&mut self) {
        for (current_window_idx, window) in self.windows.iter_mut().enumerate() {
            for (placeholder_index, block) in window.iter_mut().enumerate() {
                let placeholder = Arc::clone(&self.placeholder);
                let block = block.take().unwrap();
                let tx = self.tx.clone();
                let display_window_index = Arc::clone(&self.window_index);

                thread::spawn(move || {
                    let mut i = 0;
                    loop {
                        if *display_window_index.lock().unwrap() != current_window_idx as u32 {
                            thread::sleep(block.interval);
                            continue;
                        }

                        let start = Instant::now();
                        let output = (block.f)(i);

                        // prevents panic if the output took way too long
                        if *display_window_index.lock().unwrap() != current_window_idx as u32 {
                            thread::sleep(block.interval);
                            continue;
                        }

                        placeholder.lock().unwrap()[placeholder_index] = output;
                        tx.send(()).unwrap();

                        let time = start.elapsed();
                        if block.interval >= time {
                            thread::sleep(block.interval - time);
                        }

                        i += 1;
                    }
                });
            }
        }

        let max_window_idx = self.windows.len();
        let window_index = Arc::clone(&self.window_index);
        let placeholder = Arc::clone(&self.placeholder);
        let window_sizes = self.windows.iter().map(|i| i.len()).collect::<Vec<_>>();
        let tx = self.tx.clone();

        thread::spawn(move || {
            let mut previous_state = vec![vec![]; max_window_idx + 1];
            let _ = std::fs::remove_file(SOCKET_PATH);
            let listener = UnixListener::bind(SOCKET_PATH).unwrap();

            loop {
                println!("Listening for connection");
                for mut stream in listener.incoming().flatten() {
                    let mut buf = String::new();

                    if stream.read_to_string(&mut buf).is_err() {
                        eprintln!("[SocketError] Failed to read");
                        continue;
                    };

                    let Ok(window_idx) = buf.trim().parse::<u32>() else {
                        eprintln!("[SocketError] Invalid input received");
                        continue;
                    };

                    if window_idx > max_window_idx as u32
                        || window_sizes.len() <= window_idx as usize
                    {
                        eprintln!("[SocketError] Invalid window index");
                        continue;
                    }

                    let mut wi = window_index.lock().unwrap();
                    let previous_window_idx = *wi;
                    *wi = window_idx;
                    drop(wi);

                    let mut placeholder = placeholder.lock().unwrap();

                    // save state before switching to next window
                    previous_state[previous_window_idx as usize] = placeholder.clone();

                    let ps = &mut previous_state[window_idx as usize];
                    *placeholder = if !ps.is_empty() {
                        ps.to_vec()
                    } else {
                        vec![String::from("loading..."); window_sizes[window_idx as usize]]
                    };

                    tx.send(()).unwrap();
                }
            }
        });

        while self.rx.recv().is_ok() {
            self.update_status();
        }
    }

    fn update_status(&self) {
        let output = self.placeholder.lock().unwrap().join(self.separator);
        // std::thread::spawn(|| {
        std::process::Command::new("xsetroot")
            .arg("-name")
            .arg(output)
            .spawn()
            .unwrap()
            .wait()
            .expect("failed on wait");
        // });
    }
}
