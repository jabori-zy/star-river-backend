#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::fmt;

/// Language enum
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

/// Log parameter trait - provides type-safe parameter handling
pub trait LogParams: Clone + fmt::Debug {
    /// Apply parameters to template string and return formatted result
    fn apply_to_template(&self, template: &str) -> String;
}

/// Log message trait - unified log message interface
pub trait LogMessage: fmt::Debug + Clone + Send + Sync {
    type Params: LogParams;

    /// Get message template
    fn template(&self) -> &'static LogTemplate;
    /// Get message parameters
    fn params(&self) -> &Self::Params;
    /// Format message with specified language
    fn format_with_lang(&self, lang: Language) -> String;
    /// Convenience method: format to English
    fn to_english(&self) -> String {
        self.format_with_lang(Language::English)
    }
    /// Convenience method: format to Chinese
    fn to_chinese(&self) -> String {
        self.format_with_lang(Language::Chinese)
    }
}

/// High-performance log template
#[derive(Debug)]
pub struct LogTemplate {
    pub en: &'static str,
    pub zh: &'static str,
}

impl LogTemplate {
    /// Create new log template
    pub const fn new(en: &'static str, zh: &'static str) -> Self {
        Self { en, zh }
    }

    /// Select template based on language
    pub fn get_template(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => self.en,
            Language::Chinese => self.zh,
        }
    }

    /// Format template with parameters
    pub fn format<P: LogParams>(&self, lang: Language, params: &P) -> String {
        let template = self.get_template(lang);
        params.apply_to_template(template)
    }
}

/// Lazy log message - only format when truly needed
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

    /// Force format message
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

/// High-performance string formatter - avoid multiple string allocations
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

    /// Format template using preallocated buffer
    pub fn format_template(&mut self, template: &str, replacements: &[(&str, &dyn fmt::Display)]) -> String {
        self.buffer.clear();

        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // Escaped {{
                    chars.next(); // Consume second {
                    self.buffer.push('{');
                    continue;
                }

                // Find closing bracket - optimize with string pool
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
                    // Find and replace parameter
                    if let Some((_, value)) = replacements.iter().find(|(k, _)| *k == key) {
                        use std::fmt::Write;
                        write!(self.buffer, "{}", value).unwrap();
                    } else {
                        // Parameter not found, keep as is
                        self.buffer.push('{');
                        self.buffer.push_str(&key);
                        self.buffer.push('}');
                    }
                } else {
                    // Closing bracket not found, keep as is
                    self.buffer.push('{');
                    self.buffer.push_str(&key);
                }

                // Return key string to pool for reuse
                return_pooled_string(key);
            } else if ch == '}' && chars.peek() == Some(&'}') {
                // Escaped }}
                chars.next(); // Consume second }
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

/// String pool - reuse string objects to reduce allocations
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
                    // Limit pool size
                    pool.push(s);
                }
            }
        }
    }
}

/// Global string pool instance
static STRING_POOL: once_cell::sync::Lazy<StringPool> = once_cell::sync::Lazy::new(|| StringPool::new(1024));

/// Convenience function: get string from pool
pub fn get_pooled_string() -> String {
    STRING_POOL.get()
}

/// Convenience function: return string to pool
pub fn return_pooled_string(s: String) {
    STRING_POOL.return_string(s);
}

/// Simplified log message macro - for quickly creating single message types
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

            /// Create lazy English message
            pub fn lazy_en(self) -> LazyLogMessage<Self> {
                LazyLogMessage::english(self)
            }

            /// Create lazy Chinese message
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
