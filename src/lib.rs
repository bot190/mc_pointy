mod replacement_definitions;

pub use crate::replacement_definitions::replacements;
use std::collections::HashMap;

pub fn convert_json_dirty(values: serde_json::Value) -> replacements::Replacements
{
    let mut replace_obj: replacements::Replacements = replacements::Replacements {
        blocks: HashMap::new()
    };

    for (block_id, block_data) in values.as_object().unwrap().iter() {
        let mut block_data_objs: HashMap<replacements::ValOrAny, replacements::BlockData> = HashMap::new();

        // Create Block Data objects
        for (data_num, block_replacement) in block_data.as_object().unwrap().iter() {
            let mut block_replacement_objs: Vec<replacements::BlockReplacement> = Vec::new();

            // This is gross, but we need to figure out if there are two levels of objects here or just one
            if block_replacement.as_object().unwrap().values().next().unwrap().is_object() {
                // If the block ID object has an object inside it, then there must be multiple NBT matches, add them all
                for (replace_title, named_replace) in block_replacement.as_object().unwrap().iter() {

                    block_replacement_objs.push(replacements::BlockReplacement::new(Option::Some(replace_title.to_string()), named_replace));
                }
            } else {
                block_replacement_objs.push(replacements::BlockReplacement::new(Option::None, block_replacement));
            }

            let key = match data_num.as_str() {
                "*" => replacements::ValOrAny::Any,
                n => replacements::ValOrAny::Val(n.parse::<i64>().unwrap()),
            };

            block_data_objs.insert(key, replacements::BlockData {
                data: key,
                block_replacement: block_replacement_objs,
            });
        } // End block data objects

        let key = match block_id.as_str() {
            "*" => replacements::ValOrAny::Any,
            n => replacements::ValOrAny::Val(n.parse::<i64>().unwrap()),
        };

        replace_obj.blocks.insert(key, replacements::BlockId {
            id: key,
            block_data: block_data_objs,
        });
    }
    replace_obj
}
