// #![allow(unused_imports)]
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use tokio::sync::Mutex;
use tracing::instrument;
use uuid::Uuid;

// Normal task type definition
type NormalFunction = Box<dyn Fn() + Send + Sync + 'static>;
// Async task type definition
type AsyncFunction = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct Heartbeat {
    count: Arc<AtomicU64>,
    max_count: u64,
    interval: u64, // Heartbeat interval in milliseconds
    all_tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
    running: Arc<AtomicBool>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Task {
    NormalTask(NormalTaskConfig),
    AsyncTask(AsyncTaskConfig),
}

struct NormalTaskConfig {
    task_name: String,
    function: NormalFunction,
    interval: u64,
}

impl Debug for NormalTaskConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NormalTaskConfig {{ task_name: {}, interval: {} }}",
            self.task_name, self.interval
        )
    }
}

struct AsyncTaskConfig {
    task_name: String,
    function: AsyncFunction,
    interval: u64,
}

impl Debug for AsyncTaskConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AsyncTaskConfig {{ task_name: {}, interval: {} }}",
            self.task_name, self.interval
        )
    }
}

impl Heartbeat {
    pub fn new(interval: u64) -> Self {
        Self {
            count: Arc::new(AtomicU64::new(0)),
            max_count: 500,
            interval,
            all_tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            tracing::warn!("Heartbeat is already running");
            return Ok(());
        }

        self.running.store(true, Ordering::Relaxed);

        let count = Arc::clone(&self.count);
        let all_tasks = Arc::clone(&self.all_tasks);
        let interval = self.interval;
        let max_count = self.max_count;

        tokio::spawn(async move {
            // Create a timer with interval
            let mut tick_interval = tokio::time::interval(Duration::from_millis(interval));

            loop {
                tick_interval.tick().await;
                // let current_count = count.fetch_add(1, Ordering::SeqCst);
                let current_count = count
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
                        if count < max_count { Some(count + 1) } else { Some(0) }
                    })
                    .unwrap();
                // tracing::debug!("Heartbeat #{}", current_count);
                Heartbeat::do_task(&all_tasks, current_count).await;
            }
        });

        Ok(())
    }

    pub fn get_heartbeat_status(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Register a new async task
    ///
    /// # Arguments
    /// * `task_name` - Task name
    /// * `function` - Task function to execute
    /// * `interval` - Task execution interval (in heartbeat count units)
    ///
    /// # Returns
    /// Returns the unique identifier of the task
    pub async fn register_async_task<F, Fut>(&mut self, task_name: String, function: F, interval: u64) -> Uuid
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // Generate a task id
        let task_id = Uuid::new_v4();
        tracing::info!("Register async task: {} {}", task_id, task_name);

        let async_task_config = AsyncTaskConfig {
            task_name,
            function: Box::new(move || Box::pin(function())),
            interval,
        };
        // Generate task config
        let task_config = Task::AsyncTask(async_task_config);

        // Insert task config to task list
        self.all_tasks.lock().await.insert(task_id.clone(), task_config);

        // Return task id
        task_id
    }

    pub async fn unregister_task(&mut self, task_id: Uuid) -> Result<(), String> {
        let mut all_tasks = self.all_tasks.lock().await;
        let task = all_tasks.get(&task_id).unwrap();
        let task_name = match task {
            Task::NormalTask(task_config) => task_config.task_name.clone(),
            Task::AsyncTask(task_config) => task_config.task_name.clone(),
        };
        all_tasks.remove(&task_id);
        tracing::info!("Unregister task successfully: {} {}", task_id, task_name);
        Ok(())
    }

    async fn do_task(all_tasks: &Arc<Mutex<HashMap<Uuid, Task>>>, count: u64) {
        for (_, task) in all_tasks.lock().await.iter_mut() {
            match task {
                Task::NormalTask(task_config) => {
                    if count % task_config.interval == 0 {
                        (task_config.function)();
                    }
                }
                Task::AsyncTask(task_config) => {
                    if count % task_config.interval == 0 {
                        // tracing::info!("Execute async task: {}", task_config.task_name);
                        let future = (task_config.function)();
                        tokio::spawn(async move {
                            future.await;
                        });
                    }
                }
            }
        }
    }

    #[instrument(skip(self, function))]
    pub async fn run_async_task_once<F>(&self, task_name: String, function: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tracing::info!(task_name=%task_name, "run async task once");
        tokio::spawn(async move {
            // tracing::info!("Execute async task: {}", task_name);
            function.await;
        });
    }
}
