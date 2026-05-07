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
