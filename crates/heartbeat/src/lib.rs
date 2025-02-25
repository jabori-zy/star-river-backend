// #![allow(unused_imports)]
#![allow(dead_code)]
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::fmt::{Debug, Formatter};


// 普通任务类型定义
type NormalFunction = Box<dyn Fn() + Send + Sync + 'static>;
// 异步任务类型定义
type AsyncFunction =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct Heartbeat {
    count: Arc<AtomicU64>,
    max_count: u64,
    interval: u64,
    all_tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
    running: Arc<AtomicBool>,

}



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
        write!(f, "NormalTaskConfig {{ task_name: {}, interval: {} }}", self.task_name, self.interval)
    }
}


struct AsyncTaskConfig {
    task_name: String,
    function: AsyncFunction,
    interval: u64,
}

impl Debug for AsyncTaskConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncTaskConfig {{ task_name: {}, interval: {} }}", self.task_name, self.interval)
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

    pub async fn start(&mut self) -> Result<(), String> {
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
            // 创建一个间隔为 interval 的定时器
            let mut tick_interval =
                tokio::time::interval(Duration::from_millis(interval));

            loop {
                tick_interval.tick().await;
                // let current_count = count.fetch_add(1, Ordering::SeqCst);
                let current_count = count.fetch_update(
                    Ordering::SeqCst, 
                    Ordering::SeqCst,
                    |count| {
                        if count < max_count {
                            Some(count + 1)
                        } else {
                            Some(0)
                        }
                    }
                ).unwrap();
                tracing::trace!("Heartbeat #{}", current_count);
                Heartbeat::do_task(&all_tasks, current_count).await;


            }
        });

        Ok(())
    }

    pub fn get_heartbeat_status(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }



    /// 注册一个新的异步任务
    ///
    /// # Arguments
    /// * `task_name` - 任务名称
    /// * `function` - 要执行的任务函数
    /// * `interval` - 任务执行间隔（以心跳次数为单位）
    ///
    /// # Returns
    /// 返回任务的唯一标识符
    pub async fn register_async_task<F, Fut>(
        &mut self,
        task_name: String,
        function: F,
        interval: u64,
    ) -> Uuid
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // 生成一个任务id
        let task_id = Uuid::new_v4();
        tracing::info!("注册异步任务: {} {}", task_id, task_name);


        let async_task_config = AsyncTaskConfig {
            task_name,
            function: Box::new(move || {Box::pin(function())}),
            interval,
        };
        // 生成一个任务配置
        let task_config = Task::AsyncTask(async_task_config);

        // 将任务配置插入到任务列表中
        self.all_tasks
            .lock()
            .await
            .insert(task_id.clone(), task_config);

        // 返回任务id
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
        tracing::info!("注销任务成功: {} {}", task_id, task_name);
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
                        let future = (task_config.function)();
                        tokio::spawn(async move {
                            future.await;
                        });
                    }
                }


            }
        }
    }

    pub async fn run_async_task_once<F>(&self, task_name: String, function: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {

        tokio::spawn(async move {
            tracing::info!("执行异步任务: {}", task_name);
            function.await;
        });
    }


}
