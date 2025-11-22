#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::fmt;

/// 语言枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

/// 日志参数trait - 提供类型安全的参数处理
pub trait LogParams: Clone + fmt::Debug {
    /// 应用参数到模板字符串，返回格式化结果
    fn apply_to_template(&self, template: &str) -> String;
}

/// 日志消息trait - 统一的日志消息接口
pub trait LogMessage: fmt::Debug + Clone + Send + Sync {
    type Params: LogParams;

    /// 获取消息模板
    fn template(&self) -> &'static LogTemplate;
    /// 获取消息参数
    fn params(&self) -> &Self::Params;
    /// 使用指定语言格式化消息
    fn format_with_lang(&self, lang: Language) -> String;
    /// 便捷方法：格式化为英文
    fn to_english(&self) -> String {
        self.format_with_lang(Language::English)
    }
    /// 便捷方法：格式化为中文
    fn to_chinese(&self) -> String {
        self.format_with_lang(Language::Chinese)
    }
}

/// 高性能日志模板
#[derive(Debug)]
pub struct LogTemplate {
    pub en: &'static str,
    pub zh: &'static str,
}

impl LogTemplate {
    /// 创建新的日志模板
    pub const fn new(en: &'static str, zh: &'static str) -> Self {
        Self { en, zh }
    }

    /// 根据语言选择模板
    pub fn get_template(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => self.en,
            Language::Chinese => self.zh,
        }
    }

    /// 使用参数格式化模板
    pub fn format<P: LogParams>(&self, lang: Language, params: &P) -> String {
        let template = self.get_template(lang);
        params.apply_to_template(template)
    }
}

/// 惰性日志消息 - 只在真正需要时格式化
pub struct LazyLogMessage<M: LogMessage> {
    message: M,
    lang: Language,
}

impl<M: LogMessage> LazyLogMessage<M> {
    pub fn new(message: M, lang: Language) -> Self {
        Self { message, lang }
    }

    pub fn english(message: M) -> Self {
        Self::new(message, Language::English)
    }

    pub fn chinese(message: M) -> Self {
        Self::new(message, Language::Chinese)
    }

    /// 强制格式化消息
    pub fn format(&self) -> String {
        self.message.format_with_lang(self.lang)
    }
}

impl<M: LogMessage> fmt::Display for LazyLogMessage<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl<M: LogMessage> fmt::Debug for LazyLogMessage<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyLogMessage")
            .field("message", &self.message)
            .field("lang", &self.lang)
            .finish()
    }
}

/// 高性能字符串格式化器 - 避免多次字符串分配
pub struct FastFormatter {
    buffer: String,
}

impl FastFormatter {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(256),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    /// 使用预分配缓冲区格式化模板
    pub fn format_template(&mut self, template: &str, replacements: &[(&str, &dyn fmt::Display)]) -> String {
        self.buffer.clear();

        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // 转义的 {{
                    chars.next(); // 消费第二个 {
                    self.buffer.push('{');
                    continue;
                }

                // 寻找闭合括号 - 使用字符串池优化
                let mut key = get_pooled_string();
                let mut found_closing = false;

                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        found_closing = true;
                        break;
                    }
                    key.push(ch);
                }

                if found_closing {
                    // 查找并替换参数
                    if let Some((_, value)) = replacements.iter().find(|(k, _)| *k == key) {
                        use std::fmt::Write;
                        write!(self.buffer, "{}", value).unwrap();
                    } else {
                        // 参数未找到，保持原样
                        self.buffer.push('{');
                        self.buffer.push_str(&key);
                        self.buffer.push('}');
                    }
                } else {
                    // 没有找到闭合括号，保持原样
                    self.buffer.push('{');
                    self.buffer.push_str(&key);
                }

                // 将key字符串返回给池以重用
                return_pooled_string(key);
            } else if ch == '}' && chars.peek() == Some(&'}') {
                // 转义的 }}
                chars.next(); // 消费第二个 }
                self.buffer.push('}');
            } else {
                self.buffer.push(ch);
            }
        }

        self.buffer.clone()
    }
}

impl Default for FastFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// 字符串池 - 重用字符串对象减少分配
pub struct StringPool {
    pool: std::sync::Mutex<Vec<String>>,
    max_capacity: usize,
}

impl StringPool {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            pool: std::sync::Mutex::new(Vec::new()),
            max_capacity,
        }
    }

    pub fn get(&self) -> String {
        self.pool.lock().unwrap().pop().unwrap_or_else(|| String::with_capacity(256))
    }

    pub fn return_string(&self, mut s: String) {
        if s.capacity() <= self.max_capacity {
            s.clear();
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < 32 {
                    // 限制池大小
                    pool.push(s);
                }
            }
        }
    }
}

/// 全局字符串池实例
static STRING_POOL: once_cell::sync::Lazy<StringPool> = once_cell::sync::Lazy::new(|| StringPool::new(1024));

/// 便捷函数：从池中获取字符串
pub fn get_pooled_string() -> String {
    STRING_POOL.get()
}

/// 便捷函数：将字符串返回到池中
pub fn return_pooled_string(s: String) {
    STRING_POOL.return_string(s);
}

/// 简化的日志消息宏 - 用于快速创建单个消息类型
#[macro_export]
macro_rules! log_message {
    (
        $name:ident,
        params: ($($param:ident: $param_type:ty),* $(,)?),
        en: $en_template:literal,
        zh: $zh_template:literal
    ) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            $(pub $param: $param_type,)*
        }

        impl LogParams for $name {
            fn apply_to_template(&self, template: &str) -> String {
                let mut formatter = FastFormatter::new();
                let replacements: &[(&str, &dyn std::fmt::Display)] = &[
                    $((stringify!($param), &self.$param),)*
                ];
                formatter.format_template(template, replacements)
            }
        }

        paste::paste! {
            pub const [<$name:upper _TEMPLATE>]: LogTemplate = LogTemplate::new($en_template, $zh_template);
        }

        impl LogMessage for $name {
            type Params = Self;

            fn template(&self) -> &'static LogTemplate {
                paste::paste! { &[<$name:upper _TEMPLATE>] }
            }

            fn params(&self) -> &Self::Params {
                self
            }

            fn format_with_lang(&self, lang: Language) -> String {
                paste::paste! {
                    [<$name:upper _TEMPLATE>].format(lang, self)
                }
            }
        }

        impl $name {
            pub fn new($($param: $param_type),*) -> Self {
                Self { $($param),* }
            }

            /// 创建惰性英文消息
            pub fn lazy_en(self) -> LazyLogMessage<Self> {
                LazyLogMessage::english(self)
            }

            /// 创建惰性中文消息
            pub fn lazy_zh(self) -> LazyLogMessage<Self> {
                LazyLogMessage::chinese(self)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_english())
            }
        }
    };
}
