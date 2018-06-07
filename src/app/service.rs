use app::RestartPolicy;

#[derive(Serialize, Deserialize)]
pub struct Service {
    pub context: String,
    #[serde(default)]
    pub restart: RestartPolicy,
}
