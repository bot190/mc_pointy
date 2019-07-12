pub mod replacements {
    use serde::{Deserialize, Serialize, Serializer, Deserializer};
    use std::collections::HashMap;
    // This contains replacement information for a block id

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ValOrAny {
        Val(i64),
        Any
    }

    impl Serialize for ValOrAny {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer
        {
            let n_str = match *self {
                ValOrAny::Val(n) => n.to_string(),
                ValOrAny::Any => String::from("*"),
            };
            serializer.serialize_str(n_str.as_str())
        }
    }

    impl<'de> Deserialize<'de> for ValOrAny {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: Deserializer<'de>
        {
            let s = String::deserialize(deserializer)?;
            Ok(match s.as_str() {
                "*" => ValOrAny::Any,
                n => ValOrAny::Val(n.parse::<i64>().unwrap()),
            })
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct BlockId {
        pub id: ValOrAny,
        pub block_data: HashMap<ValOrAny, BlockData>,
    }

    // This contains replacement information for a given block data
    #[derive(Serialize, Deserialize, Debug)]
    pub struct BlockData {
        pub data: ValOrAny,
        pub block_replacement: Vec<BlockReplacement>
    }

    // This contains named replacement information.
    #[derive(Serialize, Deserialize, Debug)]
    pub struct BlockReplacement {
        title: String,
        #[serde(rename="toId")]
        to_id: Option<i64>,
        #[serde(rename="toData")]
        to_data: Option<i64>,
        #[serde(rename="addData")]
        add_data: Option<i64>,
        delete: bool,
        #[serde(rename="fromNBT")]
        from_nbt: HashMap<String, serde_json::Value>,
        #[serde(rename="toNBT")]
        to_nbt: HashMap<String, serde_json::Value>,
    }

    impl BlockReplacement {
        pub fn new (title: Option<String>, block_rep_data: &serde_json::Value) -> BlockReplacement {
            let mut from_nbt: HashMap<String, serde_json::Value> = HashMap::new();
            if block_rep_data["fromNBT"].is_object() {
                 for (tag, value) in block_rep_data["fromNBT"].as_object().unwrap().iter() {
                    from_nbt.insert(tag.to_string(), value.clone());
                }
            }
            let mut to_nbt: HashMap<String, serde_json::Value> = HashMap::new();
            if block_rep_data["toNBT"].is_object() {
                for (tag, value) in block_rep_data["toNBT"].as_object().unwrap().iter() {
                    to_nbt.insert(tag.to_string(), value.clone());
                }
            }
            // If no title was provided then default to the ID and Data values
            let replace_title = match title {
                Some(s) => s,
                None => format!("{}:{}", block_rep_data["toID"], block_rep_data["toData"]),
            };

            BlockReplacement {
                title: replace_title,
                to_id: block_rep_data["toID"].as_i64(),
                to_data: block_rep_data["toData"].as_i64(),
                add_data: block_rep_data["adjustData"].as_i64(),
                delete: block_rep_data["delete"].as_bool().unwrap_or(false),
                from_nbt: from_nbt,
                to_nbt: to_nbt,
            }
        }
    }
    // This contains the vec of block IDs
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Replacements {
        pub blocks: HashMap<ValOrAny, BlockId>
    }
}