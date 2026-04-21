#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

impl Value {
    pub fn to_idot_string(&self) -> String {
        match self {
            Self::Nil => "nil".to_string(),
            Self::Bool(value) => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Self::Number(value) => {
                let mut text = format!("{value:.15}");
                while text.contains('.') && text.ends_with('0') {
                    text.pop();
                }
                if text.ends_with('.') {
                    text.pop();
                }
                text
            }
            Self::String(value) => value.clone(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(value) => *value,
            _ => true,
        }
    }
}
