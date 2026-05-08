use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::parser::{BlockParamType, Comparison, Expression, Operation, Statement};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, (String, Value)>,
    pub lists: HashMap<String, Value>,
    pub broadcasts: HashMap<String, String>,
    pub blocks: HashMap<String, Value>,
    pub comments: HashMap<String, Value>,
    pub current_costume: u32,
    pub costumes: Vec<Costume>,
    pub sounds: Vec<Sound>,
    pub volume: f64,
    pub layer_order: i32,

    // Stage-only fields
    pub tempo: Option<f64>,
    pub video_transparency: Option<f64>,
    pub video_state: Option<String>,
    pub text_to_speech_language: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub data_format: String,
    pub asset_id: String,
    pub md5ext: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    pub name: String,
    pub asset_id: String,
    pub data_format: String,
    pub format: String,
    pub rate: u32,
    pub sample_count: u32,
    pub md5ext: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BlockPrimitive {
    Number(u8, String),
    Color(u8, String),
    String(u8, String),
    Broadcast(u8, String, String),
    Variable(u8, String, String),
    List(u8, String, String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum InputValue {
    BlockId(String),
    Primitive(BlockPrimitive),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input(
    pub u8,
    pub InputValue,
    #[serde(skip_serializing_if = "Option::is_none")] pub Option<InputValue>,
);

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Field {
    Value(String),
    ValueWithId(String, String),
    ValueWithNull(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,

    pub next: Option<String>,
    pub parent: Option<String>,

    pub inputs: HashMap<String, Input>,
    pub fields: HashMap<String, Field>,

    pub shadow: bool,
    pub top_level: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<Mutation>,
}

#[derive(Clone)]
pub struct CustomBlockDef {
    pub def_id: String,
    pub prototype_id: String,
    pub params: Vec<(String, String, BlockParamType)>, // (arg_id, name, type)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mutation {
    pub tag_name: String,
    pub children: Vec<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proccode: Option<String>, // "my block %s %b"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warp: Option<bool>,

    // procedures_prototype only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentnames: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argumentdefaults: Option<String>,

    // control_stop only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hasnext: Option<bool>,
}

impl Block {
    pub fn new(opcode: &str, parent: Option<String>) -> Self {
        Self {
            opcode: opcode.to_string(),
            next: None,
            parent,
            inputs: HashMap::new(),
            fields: HashMap::new(),
            shadow: false,
            top_level: false,
            x: None,
            y: None,
            comment: None,
            mutation: None,
        }
    }

    pub fn hat(opcode: &str, x: f64, y: f64) -> Self {
        Self {
            opcode: opcode.to_string(),
            next: None,
            parent: None,
            inputs: HashMap::new(),
            fields: HashMap::new(),
            shadow: false,
            top_level: true,
            x: Some(x),
            y: Some(y),
            comment: None,
            mutation: None,
        }
    }

    pub fn with_input(mut self, name: &str, input: Input) -> Self {
        self.inputs.insert(name.to_string(), input);
        self
    }

    pub fn with_field(mut self, name: &str, field: Field) -> Self {
        self.fields.insert(name.to_string(), field);
        self
    }
}

fn uid() -> String {
    Uuid::new_v4().to_string().replace("-", "")[..20].to_string()
}

fn number_input(n: f64) -> Input {
    Input(
        1,
        InputValue::Primitive(BlockPrimitive::Number(4, n.to_string())),
        None,
    )
}

fn string_input(s: &str) -> Input {
    Input(
        1,
        InputValue::Primitive(BlockPrimitive::String(10, s.to_string())),
        None,
    )
}

fn variable_input(name: &str, id: &str) -> Input {
    Input(
        3,
        InputValue::Primitive(BlockPrimitive::Variable(
            12,
            name.to_string(),
            id.to_string(),
        )),
        Some(InputValue::Primitive(BlockPrimitive::Number(
            4,
            "0".to_string(),
        ))),
    )
}

fn broadcast_input(name: &str, id: &str) -> Input {
    Input(
        1,
        InputValue::Primitive(BlockPrimitive::Broadcast(
            11,
            name.to_string(),
            id.to_string(),
        )),
        None,
    )
}

fn expr_to_input(
    expr: &Expression,
    var_ids: &HashMap<String, String>,
    param_ids: &HashMap<String, (String, BlockParamType)>,
    blocks: &mut HashMap<String, Block>,
    parent_id: &str,
) -> Input {
    match expr {
        Expression::Number(n) => number_input(*n),
        Expression::StringLit(s) => string_input(s),
        Expression::Ident(name) => {
            if let Some((_, param_type)) = param_ids.get(name) {
                let reporter_id = uid();
                let opcode = match param_type {
                    BlockParamType::Any => "argument_reporter_string_number",
                    BlockParamType::Bool => "argument_reporter_boolean",
                };
                let mut reporter = Block::new(opcode, Some(parent_id.to_string())).with_field(
                    "VALUE",
                    Field::ValueWithId(name.clone(), "null".to_string()),
                );
                reporter.shadow = true;
                blocks.insert(reporter_id.clone(), reporter);
                Input(3, InputValue::BlockId(reporter_id), None)
            } else {
                let id = var_ids.get(name).cloned().unwrap_or_else(uid);
                variable_input(name, &id)
            }
        }
        Expression::SelfField(name) => {
            let id = var_ids.get(name).cloned().unwrap_or_else(uid);
            variable_input(name, &id)
        }
        Expression::BinOp { left, op, right } => {
            let opcode = match op {
                Operation::Add => "operator_add",
                Operation::Subtract => "operator_subtract",
                Operation::Multiply => "operator_multiply",
                Operation::Divide => "operator_divide",
                Operation::Assign => panic!("assign is not a binary operator"),
            };

            let op_id = uid();
            let left_input = expr_to_input(left, var_ids, param_ids, blocks, &op_id);
            let right_input = expr_to_input(right, var_ids, param_ids, blocks, &op_id);

            let op_block = Block::new(opcode, Some(parent_id.to_string()))
                .with_input("NUM1", left_input)
                .with_input("NUM2", right_input);

            blocks.insert(op_id.clone(), op_block);

            Input(
                3,
                InputValue::BlockId(op_id),
                Some(InputValue::Primitive(BlockPrimitive::Number(
                    4,
                    "0".to_string(),
                ))),
            )
        }
        Expression::BoolOp { left, op, right } => {
            let opcode = match op {
                Comparison::Equals => "operator_equals",
                Comparison::GreaterThan => "operator_gt",
                Comparison::LessThan => "operator_lt",
                Comparison::And => "operator_and",
                Comparison::Or => "operator_or",
            };

            let op_id = uid();

            if (matches!(op, Comparison::And | Comparison::Or))
                && (!matches!(left.as_ref(), Expression::BoolOp { .. })
                    || !matches!(right.as_ref(), Expression::BoolOp { .. }))
            {
                panic!(
                    "comparison operator requires boolean operands: {:?} and {:?}",
                    left, right
                )
            }

            let left_input = expr_to_input(left, var_ids, param_ids, blocks, &op_id);
            let right_input = expr_to_input(right, var_ids, param_ids, blocks, &op_id);

            let op_block = Block::new(opcode, Some(parent_id.to_string()))
                .with_input("OPERAND1", left_input)
                .with_input("OPERAND2", right_input);

            blocks.insert(op_id.clone(), op_block);

            Input(2, InputValue::BlockId(op_id), None)
        }
    }
}

fn emit_stmts(
    stmts: &[Statement],
    blocks: &mut HashMap<String, Block>,
    var_ids: &HashMap<String, String>,
    param_ids: &HashMap<String, (String, BlockParamType)>,
    custom_blocks: &mut HashMap<String, CustomBlockDef>,
    broadcast_ids: &HashMap<String, String>,
    parent_id: Option<String>,
) -> Option<String> {
    let mut first_id: Option<String> = None;
    let mut prev_id = parent_id;

    for stmt in stmts {
        let id = uid();
        if first_id.is_none() {
            first_id = Some(id.clone());
        }

        let block = match stmt {
            Statement::If { condition, body } => {
                let condition_input = expr_to_input(condition, var_ids, param_ids, blocks, &id);
                let body_first = emit_stmts(
                    body,
                    blocks,
                    var_ids,
                    param_ids,
                    custom_blocks,
                    broadcast_ids,
                    Some(id.clone()),
                );
                let mut block = Block::new("control_if", prev_id.clone())
                    .with_input("CONDITION", condition_input);
                if let Some(first) = body_first {
                    block =
                        block.with_input("SUBSTACK", Input(2, InputValue::BlockId(first), None));
                }
                block
            }
            Statement::IfElse {
                condition,
                body,
                else_body,
            } => {
                let condition_input = expr_to_input(condition, var_ids, param_ids, blocks, &id);
                let body_first = emit_stmts(
                    body,
                    blocks,
                    var_ids,
                    param_ids,
                    custom_blocks,
                    broadcast_ids,
                    Some(id.clone()),
                );
                let else_body_first = emit_stmts(
                    else_body,
                    blocks,
                    var_ids,
                    param_ids,
                    custom_blocks,
                    broadcast_ids,
                    Some(id.clone()),
                );
                let mut block = Block::new("control_if_else", prev_id.clone())
                    .with_input("CONDITION", condition_input);
                if let Some(first) = body_first {
                    block =
                        block.with_input("SUBSTACK", Input(2, InputValue::BlockId(first), None));
                }
                if let Some(first) = else_body_first {
                    block =
                        block.with_input("SUBSTACK2", Input(2, InputValue::BlockId(first), None));
                }
                block
            }
            Statement::Wait { length } => Block::new("control_wait", prev_id.clone())
                .with_input("DURATION", number_input(*length)),

            Statement::Broadcast { message } => {
                let msg_id = broadcast_ids.get(message).cloned().unwrap_or_else(uid);
                Block::new("event_broadcast", prev_id.clone())
                    .with_input("BROADCAST_INPUT", broadcast_input(message, &msg_id))
            }

            Statement::BlockCall { name, args } => {
                let def = custom_blocks.get(name).expect("unknown block");
                let mut block = Block::new("procedures_call", prev_id.clone());

                let arg_ids: Vec<String> = def.params.iter().map(|(id, _, _)| id.clone()).collect();
                let proccode = if def.params.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{} {}",
                        name,
                        def.params
                            .iter()
                            .map(|(_, _, t)| match t {
                                BlockParamType::Any => "%s",
                                BlockParamType::Bool => "%b",
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                };

                for ((arg_id, _, _), arg_expr) in def.params.iter().zip(args.iter()) {
                    let input = expr_to_input(arg_expr, var_ids, param_ids, blocks, &id);
                    block = block.with_input(arg_id, input);
                }

                block.mutation = Some(Mutation {
                    tag_name: "mutation".to_string(),
                    children: vec![],
                    proccode: Some(proccode),
                    argumentids: Some(serde_json::to_string(&arg_ids).unwrap()),
                    warp: Some(false),
                    argumentnames: None,
                    argumentdefaults: None,
                    hasnext: None,
                });

                block
            }

            Statement::AssignVar {
                name,
                operation,
                value,
            } => {
                let var_id = var_ids.get(name).cloned().unwrap_or_else(uid);
                let opcode = match operation {
                    Operation::Assign => "data_setvariableto",
                    _ => "data_changevariableby",
                };
                let input_value = if matches!(operation, Operation::Subtract) {
                    match value {
                        Expression::Number(n) => number_input(-n),
                        _ => {
                            let neg_id = uid();
                            let inner = expr_to_input(value, var_ids, param_ids, blocks, &neg_id);
                            let neg_block = Block::new("operator_subtract", Some(id.clone()))
                                .with_input("NUM1", number_input(0.0))
                                .with_input("NUM2", inner);
                            blocks.insert(neg_id.clone(), neg_block);
                            Input(3, InputValue::BlockId(neg_id), None)
                        }
                    }
                } else {
                    expr_to_input(value, var_ids, param_ids, blocks, &id)
                };
                Block::new(opcode, prev_id.clone())
                    .with_input("VALUE", input_value)
                    .with_field("VARIABLE", Field::ValueWithId(name.clone(), var_id))
            }

            Statement::SelfAssign {
                field,
                operation,
                value,
            } => {
                let var_id = var_ids.get(field).cloned().unwrap_or_else(uid);
                let opcode = match operation {
                    Operation::Assign => "data_setvariableto",
                    _ => "data_changevariableby",
                };
                let input_value = if matches!(operation, Operation::Subtract) {
                    match value {
                        Expression::Number(n) => number_input(-n),
                        _ => {
                            let neg_id = uid();
                            let inner = expr_to_input(value, var_ids, param_ids, blocks, &neg_id);
                            let neg_block = Block::new("operator_subtract", Some(id.clone()))
                                .with_input("NUM1", number_input(0.0))
                                .with_input("NUM2", inner);
                            blocks.insert(neg_id.clone(), neg_block);
                            Input(3, InputValue::BlockId(neg_id), None)
                        }
                    }
                } else {
                    expr_to_input(value, var_ids, param_ids, blocks, &id)
                };
                Block::new(opcode, prev_id.clone())
                    .with_input("VALUE", input_value)
                    .with_field("VARIABLE", Field::ValueWithId(field.clone(), var_id))
            }

            Statement::VarDecl { .. }
            | Statement::Sprite { .. }
            | Statement::OnFlag { .. }
            | Statement::OnMessage { .. }
            | Statement::BlockDef { .. } => continue,
        };

        // Patch previous block's next pointer
        if let Some(prev) = &prev_id {
            if let Some(prev_block) = blocks.get_mut(prev) {
                prev_block.next = Some(id.clone());
            }
        }

        blocks.insert(id.clone(), block);
        prev_id = Some(id);
    }

    first_id
}

pub fn compile(stmts: &[Statement]) -> Value {
    let mut global_var_names: Vec<String> = vec![];
    let mut global_blocks: HashMap<String, CustomBlockDef> = HashMap::new();
    let mut broadcast_names: Vec<String> = vec![];
    let mut sprites: Vec<(&str, &[Statement])> = vec![];

    for stmt in stmts {
        match stmt {
            Statement::VarDecl { name } => global_var_names.push(name.clone()),
            Statement::Broadcast { message } => broadcast_names.push(message.clone()),
            Statement::Sprite { name, body } => sprites.push((name, body)),
            Statement::BlockDef { name, params, .. } => {
                global_blocks.insert(
                    name.clone(),
                    CustomBlockDef {
                        def_id: uid(),
                        prototype_id: uid(),
                        params: params
                            .iter()
                            .map(|p| (uid(), p.name.clone(), p.param_type))
                            .collect(),
                    },
                );
            }
            _ => {}
        }
    }

    let mut global_var_ids: HashMap<String, String> = HashMap::new();
    for name in &global_var_names {
        global_var_ids.insert(name.clone(), uid());
    }

    let mut broadcast_ids: HashMap<String, String> = HashMap::new();
    broadcast_ids.insert("message1".into(), uid());

    // Stage
    let stage_variables: HashMap<String, (String, Value)> = global_var_ids
        .iter()
        .map(|(name, id)| (id.clone(), (name.clone(), json!(0))))
        .collect();

    let stage_broadcasts: HashMap<String, String> = broadcast_ids
        .iter()
        .map(|(name, id)| (id.clone(), name.clone()))
        .collect();

    let stage = Target {
        is_stage: true,
        name: "Stage".to_string(),
        variables: stage_variables,
        lists: HashMap::new(),
        broadcasts: stage_broadcasts,
        blocks: HashMap::new(),
        comments: HashMap::new(),
        current_costume: 0,
        costumes: vec![Costume {
            name: "backdrop1".to_string(),
            data_format: "svg".to_string(),
            asset_id: "cd21514d0531fdffb22204e0ec5ed84a".to_string(),
            md5ext: "cd21514d0531fdffb22204e0ec5ed84a.svg".to_string(),
            rotation_center_x: 240.0,
            rotation_center_y: 180.0,
        }],
        sounds: vec![],
        volume: 100.0,
        layer_order: 0,
        tempo: Some(60.0),
        video_transparency: Some(50.0),
        video_state: Some("on".to_string()),
        text_to_speech_language: None,
    };

    let mut targets: Vec<Value> = vec![serde_json::to_value(stage).unwrap()];

    for (layer, (sprite_name, body)) in sprites.iter().enumerate() {
        let mut var_ids = global_var_ids.clone();
        let mut sprite_blocks: HashMap<String, CustomBlockDef> = global_blocks.clone();
        for stmt in *body {
            match stmt {
                Statement::VarDecl { name } => {
                    var_ids.insert(name.clone(), uid());
                }
                Statement::BlockDef { name, params, .. } => {
                    if !sprite_blocks.contains_key(name) {
                        sprite_blocks.insert(
                            name.clone(),
                            CustomBlockDef {
                                def_id: uid(),
                                prototype_id: uid(),
                                params: params
                                    .iter()
                                    .map(|p| (uid(), p.name.clone(), p.param_type))
                                    .collect(),
                            },
                        );
                    }
                }
                _ => {}
            }
        }

        let sprite_variables: HashMap<String, (String, Value)> = var_ids
            .iter()
            .filter(|(name, _)| !global_var_ids.contains_key(*name))
            .map(|(name, id)| (id.clone(), (name.clone(), json!(0))))
            .collect();

        let mut blocks: HashMap<String, Block> = HashMap::new();

        for stmt in *body {
            match stmt {
                Statement::OnFlag { body } => {
                    let hat_id = uid();
                    let body_first = emit_stmts(
                        body,
                        &mut blocks,
                        &var_ids,
                        &HashMap::new(),
                        &mut sprite_blocks,
                        &broadcast_ids,
                        Some(hat_id.clone()),
                    );
                    let mut hat = Block::hat("event_whenflagclicked", 0.0, (layer as f64) * 300.0);
                    hat.next = body_first;
                    blocks.insert(hat_id, hat);
                }

                Statement::OnMessage { message, body } => {
                    let hat_id = uid();
                    let msg_id = broadcast_ids.get(message).cloned().unwrap_or_else(uid);
                    let body_first = emit_stmts(
                        body,
                        &mut blocks,
                        &var_ids,
                        &HashMap::new(),
                        &mut sprite_blocks,
                        &broadcast_ids,
                        Some(hat_id.clone()),
                    );
                    let mut hat =
                        Block::hat("event_whenbroadcastreceived", 300.0, (layer as f64) * 300.0);
                    hat.next = body_first;
                    hat.fields.insert(
                        "BROADCAST_OPTION".to_string(),
                        Field::ValueWithId(message.clone(), msg_id),
                    );
                    blocks.insert(hat_id, hat);
                }

                Statement::BlockDef {
                    name,
                    params: _,
                    body,
                } => {
                    let def = sprite_blocks.get(name).expect("unknown block");
                    let hat_id = def.def_id.clone();
                    let proto_id = def.prototype_id.clone();

                    let param_ids: HashMap<String, (String, BlockParamType)> = def
                        .params
                        .iter()
                        .map(|(arg_id, name, t)| (name.clone(), (arg_id.clone(), *t)))
                        .collect();
                    let arg_ids: Vec<String> =
                        def.params.iter().map(|(id, _, _)| id.clone()).collect();
                    let arg_names: Vec<String> =
                        def.params.iter().map(|(_, name, _)| name.clone()).collect();
                    let proccode = if def.params.is_empty() {
                        name.clone()
                    } else {
                        format!(
                            "{} {}",
                            name,
                            def.params
                                .iter()
                                .map(|(_, _, t)| match t {
                                    BlockParamType::Any => "%s",
                                    BlockParamType::Bool => "%b",
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    };

                    // emit a reporter block for each param
                    let mut proto_block = Block::new("procedures_prototype", Some(hat_id.clone()));
                    proto_block.shadow = true;

                    let arg_defaults = def
                        .params
                        .iter()
                        .map(|(_, _, t)| match t {
                            BlockParamType::Any => json!(""),
                            BlockParamType::Bool => json!(false),
                        })
                        .collect::<Vec<_>>();
                    proto_block.mutation = Some(Mutation {
                        tag_name: "mutation".to_string(),
                        children: vec![],
                        proccode: Some(proccode),
                        argumentids: Some(serde_json::to_string(&arg_ids).unwrap()),
                        warp: Some(false),
                        argumentnames: Some(serde_json::to_string(&arg_names).unwrap()),
                        argumentdefaults: Some(serde_json::to_string(&arg_defaults).unwrap()),
                        hasnext: None,
                    });
                    blocks.insert(proto_id.clone(), proto_block);

                    let body_first = emit_stmts(
                        body,
                        &mut blocks,
                        &var_ids,
                        &param_ids,
                        &mut sprite_blocks,
                        &mut broadcast_ids,
                        Some(hat_id.clone()),
                    );
                    let mut hat =
                        Block::hat("procedures_definition", 600.0, (layer as f64) * 300.0);
                    hat.next = body_first;
                    hat.inputs.insert(
                        "custom_block".to_string(),
                        Input(2, InputValue::BlockId(proto_id), None),
                    );
                    blocks.insert(hat_id, hat);
                }

                _ => {}
            }
        }

        // Serialize blocks to HashMap<String, Value>
        let blocks_value: HashMap<String, Value> = blocks
            .into_iter()
            .map(|(id, block)| (id, serde_json::to_value(block).unwrap()))
            .collect();

        let sprite = Target {
            is_stage: false,
            name: sprite_name.to_string(),
            variables: sprite_variables,
            lists: HashMap::new(),
            broadcasts: HashMap::new(),
            blocks: blocks_value,
            comments: HashMap::new(),
            current_costume: 0,
            costumes: vec![Costume {
                name: "costume1".to_string(),
                data_format: "svg".to_string(),
                asset_id: "bcf454acf82e4ae9ade9109872f3766e".to_string(),
                md5ext: "bcf454acf82e4ae9ade9109872f3766e.svg".to_string(),
                rotation_center_x: 48.0,
                rotation_center_y: 50.0,
            }],
            sounds: vec![],
            volume: 100.0,
            layer_order: (layer + 1) as i32,
            tempo: None,
            video_transparency: None,
            video_state: None,
            text_to_speech_language: None,
        };

        targets.push(serde_json::to_value(sprite).unwrap());
    }

    json!({
        "targets": targets,
        "monitors": [],
        "extensions": [],
        "meta": {
            "semver": "3.0.0",
            "vm": "0.2.0",
            "agent": "ts-compiler"
        }
    })
}
