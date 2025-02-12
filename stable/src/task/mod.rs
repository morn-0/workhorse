//! 异步任务管理器
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::{
    future::{FusedFuture, FutureExt},
    pin_mut, select,
};
use once_cell::sync::Lazy;
use parking_lot::{Condvar, Mutex};
use std::future::Future;
use std::pin::Pin;
use tokio::runtime::Runtime;

pub mod executor;
pub mod manager;
pub mod signal;

pub use executor::TaskExecutor;
pub use manager::{SpawnEssentialTaskHandle, SpawnTaskHandle, TaskManager};

type TracingUnboundedSender<T> = UnboundedSender<T>;
type TracingUnboundedReceiver<T> = UnboundedReceiver<T>;

type JoinFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
type SomeFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// 任务运行限制，比如限制同时运行的任务数量
struct TaskCondition(Mutex<usize>, Condvar);

static RUNTIME: Lazy<Runtime> = Lazy::new(build_multi_thread);

pub fn handle() -> &'static tokio::runtime::Handle {
    RUNTIME.handle()
}

impl TaskCondition {
    pub fn new() -> Self {
        TaskCondition(Mutex::new(0), Condvar::new())
    }

    /// 运行数加1
    pub fn inc(&self) {
        let mut count = self.0.lock();
        *count = *count + 1;
        self.1.notify_all();
    }

    /// 运行数减1
    pub fn dec(&self) {
        let mut count = self.0.lock();
        *count = *count - 1;
        self.1.notify_all();
    }

    /// 检查运行条件, 不满足则同步等待
    pub fn check(&self, upper: usize) {
        let mut count = self.0.lock();
        while *count >= upper {
            self.1.wait(&mut count)
        }
    }
}

pub fn tracing_unbounded<T>() -> (TracingUnboundedSender<T>, TracingUnboundedReceiver<T>) {
    mpsc::unbounded()
}

fn build_multi_thread() -> Runtime {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    tokio::runtime::Builder::new_multi_thread()
        .on_thread_start(|| {
            let idx = COUNT.fetch_add(1, Ordering::SeqCst);
            tracing::trace!("#{} tokio thread started", idx + 1);
        })
        .on_thread_stop(|| {
            let idx = COUNT.fetch_sub(1, Ordering::SeqCst);
            tracing::trace!("#{} tokio thread stopped", idx);
        })
        .enable_all()
        .build()
        .expect("build tokio runtime failed!")
}

async fn signal_wrapper<F>(func: F)
where
    F: Future<Output = ()> + FusedFuture,
{
    #[cfg(unix)]
    let (mut t1, mut t2) = {
        use tokio::signal::unix::{signal, SignalKind};
        let mut t1 = signal(SignalKind::interrupt()).unwrap();
        let mut t2 = signal(SignalKind::terminate()).unwrap();
        (t1, t2)
    };

    #[cfg(windows)]
    let (mut t1, mut t2) = {
        let mut t1 = tokio::signal::windows::ctrl_c().unwrap();
        let mut t2 = tokio::signal::windows::ctrl_break().unwrap();
        (t1, t2)
    };

    let t1 = t1.recv().fuse();
    let t2 = t2.recv().fuse();
    let t3 = func;

    pin_mut!(t1, t2, t3);

    select! {
        _ = t1 => {
            tracing::info!("Received Ctrl-C Event.");
            // std::process::exit(0);
        },
        _ = t2 => {
            tracing::info!("Received Terminate/Ctrl-Break Event.");
            // std::process::exit(0);
        },
        res = t3 => {},
    }
}
