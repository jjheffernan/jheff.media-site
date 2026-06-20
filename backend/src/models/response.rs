use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseBody<T> {
    pub message: String,
    pub data: T,
}

impl<T> ResponseBody<T> {
    pub fn new(message: &str, data: T) -> ResponseBody<T> {
        ResponseBody {
            message: message.to_string(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_body_serializes_message_and_data() {
        let body = ResponseBody::new("SUCCESS", "user-id");
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["message"], "SUCCESS");
        assert_eq!(json["data"], "user-id");
    }
}
