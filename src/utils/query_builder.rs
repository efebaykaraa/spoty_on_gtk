pub struct QueryBuilder {
    params: Vec<(String, String)>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
        }
    }

    pub fn add_param(mut self, key: &str, value: String) -> Self {
        self.params.push((key.to_string(), value));
        self
    }

    pub fn add_u32(mut self, key: &str, value: u32) -> Self {
        self.params.push((key.to_string(), value.to_string()));
        self
    }

    pub fn add_string_vec(mut self, key: &str, values: Vec<String>) -> Self {
        if !values.is_empty() {
            self.params.push((key.to_string(), values.join(",")));
        }
        self
    }

    pub fn add_optional_f32(mut self, key: &str, value: Option<f32>) -> Self {
        if let Some(val) = value {
            self.params.push((key.to_string(), val.to_string()));
        }
        self
    }

    pub fn add_optional_i32(mut self, key: &str, value: Option<i32>) -> Self {
        if let Some(val) = value {
            self.params.push((key.to_string(), val.to_string()));
        }
        self
    }

    pub fn add_optional_u32(mut self, key: &str, value: Option<u32>) -> Self {
        if let Some(val) = value {
            self.params.push((key.to_string(), val.to_string()));
        }
        self
    }

    pub fn add_optional_string(mut self, key: &str, value: Option<String>) -> Self {
        if let Some(val) = value {
            self.params.push((key.to_string(), val));
        }
        self
    }

    pub fn build(self) -> String {
        self.params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&")
    }

    pub fn build_with_url(self, base_url: &str) -> String {
        if self.params.is_empty() {
            return base_url.to_string();
        }

        let query_string: Vec<String> = self.params
            .into_iter()
            .map(|(key, value)| format!("{}={}", 
                urlencoding::encode(&key), 
                urlencoding::encode(&value)))
            .collect();

        format!("{}?{}", base_url, query_string.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let params = QueryBuilder::new()
            .add_u32("size", 20)
            .add_string_vec("seeds", vec!["track1".to_string(), "track2".to_string()])
            .add_optional_f32("valence", Some(0.8))
            .add_optional_f32("energy", None)
            .add_optional_i32("key", Some(5))
            .build();
        
        assert_eq!(
            params,
            "size=20&seeds=track1,track2&valence=0.8&key=5"
        );
    }
}
